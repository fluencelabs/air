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

use super::CallResult;
use super::ExecutedState;
use super::KeeperError;

use thiserror::Error as ThisError;

/// Errors arose out of merging previous data with a new.
#[derive(ThisError, Debug)]
pub enum MergeError {
    /// Errors occurred when previous and current executed states are incompatible.
    #[error("previous and current data have incompatible states: '{0:?}' '{1:?}'")]
    IncompatibleExecutedStates(ExecutedState, ExecutedState),

    /// Errors occurred when previous and current call results are incompatible.
    #[error("previous and current call results are incompatible: '{0:?}' '{1:?}'")]
    IncompatibleCallResults(CallResult, CallResult),

    /// Errors occurred when executed trace contains less elements then corresponding Par has.
    #[error("executed trace has {0} elements, but {1} requires by Par")]
    ExecutedTraceTooSmall(usize, usize),

    /// Errors occurred when executed state contains not call result that was expected to see from fold result value pos.
    #[error("tried to obtain CallResult::Resolved by fold_result.value_pos position, but the actual state is {0:?}")]
    FoldPointsToNonCallResult(ExecutedState),

    /// Errors occurred when one of the fold subtrace lore doesn't contain 2 descriptors.
    #[error("fold contains {0} sublore descriptors, but 2 is expected")]
    FoldIncorrectSubtracesCount(usize),

    /// Errors bubbled from DataKeeper.
    #[error("{0}")]
    KeeperError(#[from] KeeperError),
    /*
    /// Errors occurred when ParResult.0 + ParResult.1 overflows.
    #[error("overflow is occurred while calculating the entire len occupied by executed states corresponded to current par: '{0:?}'")]
    ParLenOverflow(ParResult),

    /// Errors occurred when sum_i { FoldResult_i.subtrace_len } overflows.
    #[error("overflow is occurred while calculating the entire len occupied by executed states corresponded to current fold: '{0:?}'")]
    FoldLenOverflow(FoldResult),

    /// Errors occurred when ParResult.0 + ParResult.1 value is bigger than current subtree size.
    #[error("par '{0:?}' contains subtree size that is bigger than current one '{1}'")]
    ParSubtreeUnderflow(ParResult, usize),

    /// Errors occurred when sum_i { FoldResult_i.subtrace_len } value is bigger than current subtree size.
    #[error("fold '{0:?}' contains subtree size that is bigger than current one '{1}'")]
    FoldSubtreeUnderflow(FoldResult, usize),

    /// Errors occurred when one of the fold lores contains more then two sublores.
    #[error("fold sublores have different value_pos: {0}, {1}")]
    FoldIncorrectValuePos(usize, usize),

    #[error("value_pos of a FoldResult points to '{0}', but this element isn't an element of a stream")]
    FoldValuesPosNotStream(usize),
     */
}
