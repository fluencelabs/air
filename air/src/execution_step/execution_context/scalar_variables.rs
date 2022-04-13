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

use crate::execution_step::boxed_value::ScalarRef;
use crate::execution_step::errors_prelude::*;
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
    // TODO: use Rc<String> to avoid copying
    pub(crate) local_values: HashMap<String, Vec<SparseCell>>,
    pub(crate) iterable_values: HashMap<String, FoldState<'i>>,
    pub(crate) fold_block_id: usize,
}

pub(crate) struct SparseCell {
    /// Position in a inner fold layer where the value was set.
    pub(crate) position: usize,
    pub(crate) value: ValueAggregate,
}

impl SparseCell {
    pub(crate) fn new(position: usize, value: ValueAggregate) -> Self {
        Self { position, value }
    }
}

impl<'i> Scalars<'i> {
    /// Returns true if there was a previous value for the provided key on the same
    /// fold block.
    pub(crate) fn set_value(&mut self, name: impl Into<String>, value: ValueAggregate) -> ExecutionResult<bool> {
        use std::collections::hash_map::Entry::{Occupied, Vacant};

        let shadowing_allowed = self.shadowing_allowed();
        match self.local_values.entry(name.into()) {
            Vacant(entry) => {
                let cell = SparseCell::new(self.fold_block_id, value);
                entry.insert(vec![cell]);

                Ok(false)
            }
            Occupied(entry) => {
                if !shadowing_allowed {
                    return Err(UncatchableError::MultipleVariablesFound(entry.key().clone()).into());
                }

                let values = entry.into_mut();
                let last_cell = values.last_mut().expect("");
                if last_cell.position == self.fold_block_id {
                    last_cell.value = value;
                    Ok(true)
                } else {
                    let new_cell = SparseCell::new(self.fold_block_id, value);
                    values.push(new_cell);
                    Ok(false)
                }
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
            Occupied(entry) => Err(UncatchableError::MultipleIterableValues(entry.key().clone()).into()),
        }
    }

    pub(crate) fn remove_iterable_value(&mut self, name: &str) {
        self.iterable_values.remove(name);
    }

    pub(crate) fn get_value(&'i self, name: &str) -> ExecutionResult<&'i ValueAggregate> {
        self.local_values
            .get(name)
            .map(|values| &values.last().expect("").value)
            .ok_or_else(|| Rc::new(CatchableError::VariableNotFound(name.to_string())).into())
    }

    pub(crate) fn get_iterable_mut(&mut self, name: &str) -> ExecutionResult<&mut FoldState<'i>> {
        self.iterable_values
            .get_mut(name)
            .ok_or_else(|| UncatchableError::FoldStateNotFound(name.to_string()).into())
    }

    pub(crate) fn get(&'i self, name: &str) -> ExecutionResult<ScalarRef<'i>> {
        let value = self.get_value(name);
        let iterable_value = self.iterable_values.get(name);

        match (value, iterable_value) {
            (Err(_), None) => Err(CatchableError::VariableNotFound(name.to_string()).into()),
            (Ok(value), None) => Ok(ScalarRef::Value(value)),
            (Err(_), Some(iterable_value)) => Ok(ScalarRef::IterableValue(iterable_value)),
            (Ok(_), Some(_)) => unreachable!("this is checked on the parsing stage"),
        }
    }

    pub(crate) fn meet_scope_start(&mut self) {
        self.fold_block_id += 1;
    }

    pub(crate) fn meet_scope_end(&mut self) {
        self.fold_block_id -= 1;
        let mut values_to_delete = Vec::new();
        for (name, values) in self.local_values.iter_mut() {
            let position = values.last().expect("").position;
            if position != 0 && position >= self.fold_block_id {
                values.pop();
            }
            if values.is_empty() {
                values_to_delete.push(name.to_string());
            }
        }

        for value_name in values_to_delete {
            self.local_values.remove(&value_name);
        }
    }

    pub(crate) fn shadowing_allowed(&self) -> bool {
        // shadowing is allowed only inside a fold block, 0 here means that execution flow
        // is in a global scope
        self.fold_block_id != 0
    }
}

use std::fmt;

impl<'i> fmt::Display for Scalars<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "fold_block_id: {}", self.fold_block_id)?;

        for (name, _) in self.local_values.iter() {
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

#[cfg(test)]
mod test {
    use super::*;
    use polyplets::SecurityTetraplet;

    use serde_json::json;
    use std::rc::Rc;

    #[test]
    fn test_local_cleanup() {
        let mut scalars = Scalars::default();

        let tetraplet = SecurityTetraplet::default();
        let rc_tetraplet = Rc::new(tetraplet);
        let value = json!(1u64);
        let rc_value = Rc::new(value);
        let value_aggregate = ValueAggregate::new(rc_value, rc_tetraplet, 1);
        let value_1_name = "name_1";
        scalars.set_value(value_1_name, value_aggregate.clone()).unwrap();

        let value_2_name = "name_2";
        scalars.meet_scope_start();
        scalars.set_value(value_2_name, value_aggregate.clone()).unwrap();
        scalars.meet_scope_start();
        scalars.set_value(value_2_name, value_aggregate.clone()).unwrap();

        let expected_values_count = scalars.local_values.get(value_2_name).unwrap().len();
        assert_eq!(expected_values_count, 2);

        scalars.meet_scope_end();
        let expected_values_count = scalars.local_values.get(value_2_name).unwrap().len();
        assert_eq!(expected_values_count, 1);

        scalars.meet_scope_end();
        assert!(scalars.local_values.get(value_2_name).is_none());

        let expected_values_count = scalars.local_values.get(value_1_name).unwrap().len();
        assert_eq!(expected_values_count, 1);
    }
}
