/*
 * Copyright 2021 Fluence Labs Limited
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use crate::exec_err;
use crate::execution_step::boxed_value::ScalarRef;
use crate::execution_step::ExecutionError;
use crate::execution_step::ExecutionResult;
use crate::execution_step::FoldState;
use crate::execution_step::ValueAggregate;

use std::collections::HashMap;
use std::rc::Rc;

/// There are two scopes for variable scalars in AIR: global and local. A local scope
/// is a scope inside every fold block, other scope is a global. It means that scalar
/// in an upper fold block could be shadowed by a scalar with the same name in a lower
/// fold block, it works "as expected". Let's consider the following example:
/// (seq
///   (seq
///     (call ... local) ;; (1)
///     (fold iterable_1 iterator_1
///       (seq
///         (seq
///           (seq
///             (call ... local) ;; (2)
///             (fold iterable_2 iterator_2
///               (seq
///                 (seq
///                    (call ... local) ;; (3)
///                    (call ... [local]) ;; local set by (3) will be used
///                  )
///                  (next iterator_2)
///               )
///             )
///           )
///           (call ... [local]) ;; local set by (2) will be used
///         )
///         (next iterator_1)
///       )
///     )
///   )
///   (seq
///     (call ... [local]) ;; local set by (1) will be used
///     (call ... local) ;; error will be occurred because, it's impossible to set variable twice
///                      ;; in a global scope
///   )
/// )
///
/// Although there could be only one iterable value for a fold block, because of CRDT rules.
/// This struct is intended to provide abilities to work with scalars as it was described.
#[derive(Default)]
pub(crate) struct Scalars<'i> {
    // this one is optimized for speed (not for memory), because it's unexpected
    // that a script could have a lot of inner folds.
    pub values: HashMap<String, Vec<Option<ValueAggregate>>>,
    pub iterable_values: HashMap<String, FoldState<'i>>,
    pub fold_block_id: usize,
}

#[allow(dead_code)]
impl<'i> Scalars<'i> {
    /// Returns true if there was a previous value for the provided key on the same
    /// fold block.
    pub(crate) fn set_value(&mut self, name: impl Into<String>, value: ValueAggregate) -> ExecutionResult<bool> {
        use std::collections::hash_map::Entry::{Occupied, Vacant};

        let shadowing_allowed = self.shadowing_allowed();
        match self.values.entry(name.into()) {
            Vacant(entry) => {
                let mut values = vec![None; self.fold_block_id];
                values.push(Some(value));
                entry.insert(values);

                Ok(false)
            }
            Occupied(entry) => {
                if !shadowing_allowed {
                    return exec_err!(ExecutionError::MultipleVariablesFound(entry.key().clone()));
                }

                let values = entry.into_mut();
                let contains_prev_value = values
                    .get(self.fold_block_id)
                    .map_or_else(|| false, |value| value.is_none());
                // could be considered as lazy erasing
                values.resize(self.fold_block_id + 1, None);

                values[self.fold_block_id] = Some(value);
                Ok(contains_prev_value)
            }
        }
    }

    pub(crate) fn set_iterable_value(
        &mut self,
        name: impl Into<String>,
        fold_state: FoldState<'i>,
    ) -> ExecutionResult<()> {
        use std::collections::hash_map::Entry::{Occupied, Vacant};

        match self.iterable_values.entry(name.into()) {
            Vacant(entry) => {
                entry.insert(fold_state);
                Ok(())
            }
            Occupied(entry) => {
                exec_err!(ExecutionError::MultipleIterableValues(entry.key().clone()))
            }
        }
    }

    pub(crate) fn remove_value(&mut self, name: &str) {
        self.values.remove(name);
    }

    pub(crate) fn remove_iterable_value(&mut self, name: &str) {
        self.iterable_values.remove(name);
    }

    pub(crate) fn get_value(&'i self, name: &str) -> ExecutionResult<&'i ValueAggregate> {
        self.values
            .get(name)
            .and_then(|scalars| {
                scalars
                    .iter()
                    .take(self.fold_block_id + 1)
                    .rev()
                    .find_map(|scalar| scalar.as_ref())
            })
            .ok_or_else(|| Rc::new(ExecutionError::VariableNotFound(name.to_string())))
    }

    pub(crate) fn get_value_mut(&'i mut self, name: &str) -> ExecutionResult<&'i mut ValueAggregate> {
        let fold_block_id = self.fold_block_id;
        self.values
            .get_mut(name)
            .and_then(|scalars| {
                scalars
                    .iter_mut()
                    .take(fold_block_id)
                    .rev()
                    .find_map(|scalar| scalar.as_mut())
            })
            .ok_or_else(|| Rc::new(ExecutionError::VariableNotFound(name.to_string())))
    }

    pub(crate) fn get_iterable(&self, name: &str) -> ExecutionResult<&FoldState<'i>> {
        self.iterable_values
            .get(name)
            .ok_or_else(|| Rc::new(ExecutionError::FoldStateNotFound(name.to_string())))
    }

    pub(crate) fn get_iterable_mut(&mut self, name: &str) -> ExecutionResult<&mut FoldState<'i>> {
        self.iterable_values
            .get_mut(name)
            .ok_or_else(|| Rc::new(ExecutionError::FoldStateNotFound(name.to_string())))
    }

    pub(crate) fn get(&'i self, name: &str) -> ExecutionResult<ScalarRef<'i>> {
        let value = self.get_value(name);
        let iterable_value = self.iterable_values.get(name);

        match (value, iterable_value) {
            (Err(_), None) => exec_err!(ExecutionError::VariableNotFound(name.to_string())),
            (Ok(value), None) => Ok(ScalarRef::Value(value)),
            (Err(_), Some(iterable_value)) => Ok(ScalarRef::IterableValue(iterable_value)),
            (Ok(_), Some(_)) => unreachable!("this is checked on the parsing stage"),
        }
    }

    pub(crate) fn meet_fold_start(&mut self) {
        self.fold_block_id += 1;
    }

    pub(crate) fn meet_fold_end(&mut self) {
        self.fold_block_id -= 1;
        if self.fold_block_id == 0 {
            // lazy cleanup after exiting from a top fold block to the global scope
            self.cleanup()
        }
    }

    pub(crate) fn shadowing_allowed(&self) -> bool {
        // shadowing is allowed only inside a fold block, 0 here means that execution flow
        // is in a global scope
        self.fold_block_id != 0
    }

    fn cleanup(&mut self) {
        for (_, scalars) in self.values.iter_mut() {
            scalars.truncate(self.fold_block_id + 1)
        }
    }
}

use std::fmt;

impl<'i> fmt::Display for Scalars<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "fold_block_id: {}", self.fold_block_id)?;

        for (name, _) in self.values.iter() {
            let value = self.get_value(name);
            if let Ok(last_value) = value {
                writeln!(f, "{} => {}", name, last_value.result)?;
            }
        }

        for (name, _) in self.iterable_values.iter() {
            // it's impossible to print an iterable value for now
            writeln!(f, "{} => iterable", name)?;
        }

        Ok(())
    }
}
