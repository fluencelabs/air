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

use super::*;

const EXPECTED_STATE_NAME: &str = "ap";

#[derive(Debug, Clone)]
pub enum MergerApResult {
    /// There is no corresponding state in a trace for this call.
    Empty,

    /// There was a state in at least one of the contexts. If there were two states in
    /// both contexts, they were successfully merged.
    ApResult { res_generation: Option<u32> },
}

pub(crate) fn try_merge_next_state_as_ap<VT>(data_keeper: &mut DataKeeper<VT>) -> MergeResult<MergerApResult, VT> {
    use ExecutedState::Ap;
    use PreparationScheme::*;

    let prev_state = data_keeper.prev_slider_mut().next_state();
    let current_state = data_keeper.current_slider_mut().next_state();

    match (prev_state, current_state) {
        (Some(Ap(prev_ap)), Some(_)) => prepare_merge_result(Some(prev_ap), Both, data_keeper),
        (Some(Ap(prev_ap)), None) => prepare_merge_result(Some(prev_ap), Previous, data_keeper),
        // check that current state is Ap, but it's impossible to use it, because prev_data
        // could not have streams with such generations
        (None, Some(Ap(_))) => prepare_merge_result(None, Current, data_keeper),
        (None, None) => Ok(MergerApResult::Empty),
        (prev_state, current_state) => Err(MergeError::incompatible_states(
            prev_state,
            current_state,
            EXPECTED_STATE_NAME,
        )),
    }
}

fn prepare_merge_result<VT>(
    ap_result: Option<ApResult>,
    scheme: PreparationScheme,
    data_keeper: &mut DataKeeper<VT>,
) -> MergeResult<MergerApResult, VT> {
    prepare_positions_mapping(scheme, data_keeper);

    match ap_result {
        Some(ap_result) => to_merger_result(ap_result),
        None => Ok(MergerApResult::Empty),
    }
}

macro_rules! to_maybe_generation {
    ($ap_result:ident, $generations:expr, $error_ty:ident) => {
        match $generations.len() {
            0 => None,
            1 => Some($generations[0]),
            _ => {
                let ap_error = super::ApResultError::$error_ty($ap_result);
                return Err(super::MergeError::IncorrectApResult(ap_error));
            }
        }
    };
}

fn to_merger_result<VT>(ap_result: ApResult) -> MergeResult<MergerApResult, VT> {
    let res_generation = to_maybe_generation!(ap_result, &ap_result.res_generations, TooManyDstGenerations);

    let ap_result = MergerApResult::ApResult { res_generation };

    Ok(ap_result)
}
