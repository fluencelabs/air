/*
 * Copyright 2020 Fluence Labs Limited
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

mod air;
mod boxed_value;
mod errors;
pub(crate) mod execution_context;
mod joinable;
mod trace_handler;
mod utils;

pub(super) use self::air::ExecutableInstruction;
pub(super) use self::air::FoldState;
pub(super) use boxed_value::Generation;
pub(super) use boxed_value::ResolvedCallResult;
pub(super) use boxed_value::Scalar;
pub(super) use boxed_value::Stream;
pub(crate) use errors::Catchable;
pub(super) use errors::ExecutionError;
pub(crate) use execution_context::ExecutionCtx;
use joinable::Joinable;
pub(crate) use trace_handler::TraceHandler;

use std::cell::RefCell;
use std::rc::Rc;

type ExecutionResult<T> = std::result::Result<T, Rc<ExecutionError>>;
type RSecurityTetraplet = Rc<RefCell<crate::SecurityTetraplet>>;
type SecurityTetraplets = Vec<RSecurityTetraplet>;

use air_parser::ast::AstVariable;

#[macro_export]
macro_rules! exec_err {
    ($err:expr) => {
        Err(std::rc::Rc::new($err))
    };
}
