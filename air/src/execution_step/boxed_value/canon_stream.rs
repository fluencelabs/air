/*
 * Copyright 2022 Fluence Labs Limited
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

use super::Stream;
use super::ValueAggregate;
use crate::execution_step::Generation;
use crate::JValue;

use std::fmt::Formatter;
use std::rc::Rc;

/// Canon stream is a value type lies between a scalar and a stream, it has the same algebra as
/// scalars, and represent a stream fixed at some execution point.
#[derive(Debug, Default, Clone)]
pub struct CanonStream {
    values: Vec<ValueAggregate>,
    // tetraplet is needed to handle adding canon streams as a whole to a stream.
    tetraplet: Rc<SecurityTetraplet>,
    #[allow(dead_code)]
    position: TracePos,
}

impl CanonStream {
    pub(crate) fn new(values: Vec<ValueAggregate>, peer_pk: String, position: TracePos) -> Self {
        // tetraplet is comprised only from peer_pk here
        let tetraplet = SecurityTetraplet::new(peer_pk, "", "", "");
        Self {
            values,
            tetraplet: Rc::new(tetraplet),
            position,
        }
    }

    pub(crate) fn from_stream(stream: &Stream, peer_pk: String, position: TracePos) -> Self {
        let values = stream.iter(Generation::Last).unwrap().cloned().collect::<Vec<_>>();
        let tetraplet = SecurityTetraplet::new(peer_pk, "", "", "");
        Self {
            values,
            tetraplet: Rc::new(tetraplet),
            position,
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.values.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub(crate) fn as_jvalue(&self) -> JValue {
        use std::ops::Deref;

        // TODO: this clone will be removed after boxed values
        let jvalue_array = self.values.iter().map(|r| r.result.deref().clone()).collect::<Vec<_>>();
        JValue::Array(jvalue_array)
    }

    pub(crate) fn iter(&self) -> impl ExactSizeIterator<Item = &ValueAggregate> {
        self.values.iter()
    }

    pub(crate) fn nth(&self, idx: usize) -> Option<&ValueAggregate> {
        self.values.get(idx)
    }

    pub(crate) fn tetraplet(&self) -> &Rc<SecurityTetraplet> {
        &self.tetraplet
    }
}

use air_interpreter_data::TracePos;
use polyplets::SecurityTetraplet;
use std::fmt;

impl fmt::Display for CanonStream {
    fn fmt(&self, _f: &mut Formatter<'_>) -> fmt::Result {
        Ok(())
        //write!(f, "#[{}]", self.0.join(", "))
    }
}
