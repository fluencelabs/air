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

mod fold_iterable_state;
mod iterable;
mod jvaluable;
mod scalar;
mod stream;
mod value_aggregate;
mod variable;

pub use iterable::*;
pub use jvaluable::*;
pub use scalar::ScalarRef;
pub use scalar::ValueAggregate;
pub use stream::Generation;
pub use stream::Stream;
pub use stream::StreamIter;
pub use value_aggregate::ValueAggregate;
pub use variable::Variable;

pub(crate) use polyplets::SecurityTetraplet;

use std::rc::Rc;

type RcSecurityTetraplet = Rc<crate::SecurityTetraplet>;
type RcSecurityTetraplets = Vec<RcSecurityTetraplet>;
