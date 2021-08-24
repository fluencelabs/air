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
#![warn(rust_2018_idioms)]
#![deny(
    dead_code,
    nonstandard_style,
    unused_imports,
    unused_mut,
    unused_variables,
    unused_unsafe,
    unreachable_patterns
)]

mod avm;
mod config;
mod data_store;
mod errors;
mod runner;

pub use avm::AVM;
pub use config::AVMConfig;
pub use errors::AVMError;
pub use runner::AVMRunner;

// Re-exports
pub use fluence_faas::ne_vec;
pub use fluence_faas::Ctx;
pub use fluence_faas::HostExportedFunc;
pub use fluence_faas::HostImportDescriptor;
pub use fluence_faas::HostImportError;
pub use fluence_faas::IType;
pub use fluence_faas::IValue;

pub use air_interpreter_interface::CallResults;
pub use air_interpreter_interface::InterpreterOutcome;
pub use air_interpreter_interface::INTERPRETER_SUCCESS;

pub(crate) type AVMResult<T> = std::result::Result<T, AVMError>;
