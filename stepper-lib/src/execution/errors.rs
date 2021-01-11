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

use crate::build_targets::CallServiceResult;
use crate::call_evidence::CallResult;
use crate::call_evidence::EvidenceState;
use crate::JValue;
use crate::ResolvedCallResult;

use jsonpath_lib::JsonPathError;
use serde_json::Error as SerdeJsonError;
use thiserror::Error as ThisError;

use std::env::VarError;
use std::error::Error;

/// Errors arised while executing AIR script.
#[derive(ThisError, Debug)]
pub enum ExecutionError {
    /// Errors occurred while parsing returned by call_service value.
    #[error("call_service result '{0:?}' can't be serialized or deserialized with an error: {1:?}")]
    CallServiceResultDeError(CallServiceResult, SerdeJsonError),

    /// Semantic errors in instructions.
    #[error("{0}")]
    InstructionError(String),

    /// An error is occurred while calling local service via call_service.
    #[error("{0}")]
    LocalServiceError(String),

    /// Value for such name isn't presence in data.
    #[error("variable with name '{0}' isn't present in data")]
    VariableNotFound(String),

    /// Multiple values for such name found.
    #[error("multiple variables found for name {0} in data")]
    MultipleVariablesFound(String),

    /// An error occurred while trying to apply json path to this JValue.
    #[error("variable with path '{1}' not found in '{0:?}' with an error: '{2:?}'")]
    JValueJsonPathError(JValue, String, JsonPathError),

    /// An error occurred while trying to apply json path to this accumulator with JValue's.
    #[error("variable with path '{1}' not found in '{0:?}' with error: '{2:?}'")]
    JValueAccJsonPathError(Vec<ResolvedCallResult>, String, JsonPathError),

    /// Provided JValue has incompatible with target type.
    #[error("expected JValue type '{1}', but got '{0:?}' JValue")]
    IncompatibleJValueType(JValue, &'static str),

    /// Provided AValue has incompatible with target type.
    #[error("expected AValue type '{1}', but got '{0:?}' AValue")]
    IncompatibleAValueType(String, String),

    /// Multiple values found for such json path.
    #[error("multiple variables found for this json path '{0}'")]
    MultipleValuesInJsonPath(String),

    /// Fold state wasn't found for such iterator name.
    #[error("fold state not found for this iterable '{0}'")]
    FoldStateNotFound(String),

    /// Multiple fold states found for such iterator name.
    #[error("multiple fold states found for iterable '{0}'")]
    MultipleFoldStates(String),

    /// Expected evidence state of different type.
    #[error("invalid evidence state: expected '{0}', but actual {1:?}")]
    InvalidEvidenceState(String, ExecutedState),

    /// Errors occurred when evidence path contains less elements then corresponding Par has.
    #[error("vairable with name '{}' can't be shadowed, shadowing is supported only for scalar values")]
    ShadowingError(String),
}

impl Error for ExecutionError {}

impl From<std::convert::Infallible> for ExecutionError {
    fn from(_: std::convert::Infallible) -> Self {
        unreachable!()
    }
}
