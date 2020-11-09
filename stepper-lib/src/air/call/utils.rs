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

use super::ExecutionCtx;
use crate::call_evidence::CallEvidenceCtx;
use crate::call_evidence::CallResult;
use crate::call_evidence::EvidenceState;
use crate::log_targets::EVIDENCE_CHANGING;
use crate::AValue;
use crate::AquamarineError;
use crate::JValue;
use crate::Result;

use air_parser::ast::CallOutput;

use std::{cell::RefCell, rc::Rc};

/// Writes result of a local `Call` instruction to `ExecutionCtx` at `output`
pub(super) fn set_local_call_result<'i>(
    output: CallOutput<'i>,
    exec_ctx: &mut ExecutionCtx<'i>,
    result: Rc<JValue>,
) -> Result<()> {
    use std::collections::hash_map::Entry::{Occupied, Vacant};
    use AquamarineError::*;

    match output {
        CallOutput::Scalar(name) => {
            match exec_ctx.data_cache.entry(name.to_string()) {
                Vacant(entry) => entry.insert(AValue::JValueRef(result)),
                Occupied(entry) => return Err(MultipleVariablesFound(entry.key().clone())),
            };
        }
        CallOutput::Accumulator(name) => {
            match exec_ctx.data_cache.entry(name.to_string()) {
                Occupied(mut entry) => match entry.get_mut() {
                    // if result is an array, insert result to the end of the array
                    AValue::JValueAccumulatorRef(values) => values.borrow_mut().push(result),
                    v => return Err(IncompatibleAValueType(format!("{:?}", v), String::from("Array"))),
                },
                Vacant(entry) => {
                    entry.insert(AValue::JValueAccumulatorRef(RefCell::new(vec![result])));
                }
            };
        }
        CallOutput::None => {}
    }

    Ok(())
}

/// Writes evidence of a particle being sent to remote node
pub(super) fn set_remote_call_result<'i>(
    peer_pk: String,
    exec_ctx: &mut ExecutionCtx<'i>,
    call_ctx: &mut CallEvidenceCtx,
) {
    exec_ctx.next_peer_pks.push(peer_pk);
    exec_ctx.subtree_complete = false;

    let new_evidence_state = EvidenceState::Call(CallResult::RequestSent(exec_ctx.current_peer_id.clone()));
    log::info!(
        target: EVIDENCE_CHANGING,
        "  adding new call evidence state {:?}",
        new_evidence_state
    );
    call_ctx.new_path.push_back(new_evidence_state);
}
