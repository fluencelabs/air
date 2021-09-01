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

mod catchable;
mod joinable;

pub(crate) use catchable::Catchable;
pub(crate) use joinable::Joinable;

use super::trace_handler::MergerApResult;
use super::trace_handler::TraceHandlerError;
use super::ResolvedCallResult;
use super::Stream;
use crate::JValue;

use jsonpath_lib::JsonPathError;
use thiserror::Error as ThisError;

use std::rc::Rc;

/// Errors arised while executing AIR script.
#[derive(ThisError, Debug)]
pub(crate) enum ExecutionError {
    /// Semantic errors in a call instructions.
    #[error("call should have service id specified by peer part or function part")]
    IncorrectCallTriplet,

    /// An error is occurred while calling local service via call_service.
    /// This error type includes both service error and de service result error.
    #[error("Local service error, ret_code is {0}, error message is '{1}'")]
    LocalServiceError(i32, Rc<String>),

    /// Value for such name isn't presence in data.
    #[error("variable with name '{0}' isn't present in data")]
    VariableNotFound(String),

    /// Multiple values for such name found.
    #[error("multiple variables found for name '{0}' in data")]
    MultipleVariablesFound(String),

    /// An error occurred while trying to apply json path to this JValue.
    #[error("variable with path '{1}' not found in '{0}' with an error: '{2}'")]
    JValueJsonPathError(JValue, String, JsonPathError),

    /// An error occurred while trying to apply json path to this stream generation with JValue's.
    #[error("variable with path '{1}' not found in '{0:?}' with error: '{2}'")]
    GenerationStreamJsonPathError(Vec<ResolvedCallResult>, String, JsonPathError),

    /// An error occurred while trying to apply json path to this stream with JValue's.
    #[error("variable with path '{1}' not found in '{0:?}' with error: '{2}'")]
    StreamJsonPathError(Stream, String, JsonPathError),

    /// An error occurred while trying to apply json path to an empty stream.
    #[error("json path {0} is applied to an empty stream")]
    EmptyStreamJsonPathError(String),

    /// Provided JValue has incompatible with target type.
    #[error("expected JValue type '{1}', but got '{0}' JValue")]
    IncompatibleJValueType(JValue, &'static str),

    /// Provided AValue has incompatible with target type.
    #[error("expected AValue type '{1}', but got '{0}' AValue")]
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

    /// Errors encountered while shadowing non-scalar values.
    #[error("variable with name '{0}' can't be shadowed, shadowing isn't supported for iterables")]
    IterableShadowing(String),

    /// This error type is produced by a match to notify xor that compared values aren't equal.
    #[error("match is used without corresponding xor")]
    MatchWithoutXorError,

    /// This error type is produced by a mismatch to notify xor that compared values aren't equal.
    #[error("mismatch is used without corresponding xor")]
    MismatchWithoutXorError,

    /// This error type is produced by a mismatch to notify xor that compared values aren't equal.
    #[error("jvalue '{0}' can't be flattened, to be flattened a jvalue should have an array type and consist of zero or one values")]
    FlatteningError(JValue),

    /// Json path is applied to scalar that have inappropriate type.
    #[error(
        "json path can't be applied to scalar '{0}',\
    it could be applied only to streams and variables of array and object types"
    )]
    JsonPathVariableTypeError(JValue),

    /// Errors bubbled from a trace handler.
    #[error("{0}")]
    TraceError(#[from] TraceHandlerError),

    /// Errors occurred while insertion of a value inside stream that doesn't have corresponding generation.
    #[error("stream {0:?} doesn't have generation with number {1}, probably a supplied to the interpreter data is corrupted")]
    StreamDontHaveSuchGeneration(Stream, usize),

    /// Errors occurred when result from data doesn't match to a instruction, f.e. an instruction
    /// could be applied to a stream, but result doesn't contain generation in a source position.
    #[error("ap result {0:?} doesn't match corresponding instruction")]
    ApResultNotCorrespondToInstr(MergerApResult),
}

impl From<TraceHandlerError> for Rc<ExecutionError> {
    fn from(trace_error: TraceHandlerError) -> Self {
        Rc::new(ExecutionError::TraceError(trace_error))
    }
}

impl ExecutionError {
    pub(crate) fn to_error_code(&self) -> u32 {
        use ExecutionError::*;

        match self {
            IncorrectCallTriplet => 1,
            LocalServiceError(..) => 2,
            VariableNotFound(_) => 3,
            MultipleVariablesFound(_) => 4,
            JValueJsonPathError(..) => 5,
            GenerationStreamJsonPathError(..) => 6,
            IncompatibleJValueType(..) => 7,
            IncompatibleAValueType(..) => 8,
            MultipleValuesInJsonPath(_) => 9,
            FoldStateNotFound(_) => 10,
            MultipleFoldStates(_) => 11,
            IterableShadowing(_) => 12,
            MatchWithoutXorError => 13,
            MismatchWithoutXorError => 14,
            FlatteningError(_) => 15,
            JsonPathVariableTypeError(_) => 16,
            StreamJsonPathError(..) => 17,
            StreamDontHaveSuchGeneration(..) => 18,
            ApResultNotCorrespondToInstr(_) => 19,
            EmptyStreamJsonPathError(_) => 20,
            TraceError(_) => 21,
        }
    }
}

macro_rules! log_join {
    ($($args:tt)*) => {
        log::info!(target: crate::log_targets::JOIN_BEHAVIOUR, $($args)*)
    }
}

#[rustfmt::skip::macros(log_join)]
impl Joinable for ExecutionError {
    /// Returns true, if supplied error is related to variable not found errors type.
    /// Print log if this is joinable error type.
    fn is_joinable(&self) -> bool {
        use ExecutionError::*;

        match self {
            VariableNotFound(var_name) => {
                log_join!("  waiting for an argument with name '{}'", var_name);
                true
            }
            StreamJsonPathError(stream, json_path, _) => {
                log_join!("  waiting for an argument with path '{}' on stream '{:?}'", json_path, stream);
                true
            }
            EmptyStreamJsonPathError(json_path) => {
                log_join!("  waiting on empty stream for path '{}'", json_path);
                true
            }

            _ => false,
        }
    }
}

impl Catchable for ExecutionError {
    fn is_catchable(&self) -> bool {
        // this kind is related to an invalid data and should treat as a non-catchable error
        !matches!(self, ExecutionError::TraceError(_))
    }
}

impl From<std::convert::Infallible> for ExecutionError {
    fn from(_: std::convert::Infallible) -> Self {
        unreachable!()
    }
}
