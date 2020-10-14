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

use super::EvidenceState;

use serde::Deserialize;
use serde::Serialize;

use std::collections::VecDeque;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct CallEvidenceCtx {
    pub(crate) current_states: VecDeque<EvidenceState>,
    pub(crate) used_states_in_subtree: usize,
    pub(crate) subtree_size: usize,
    pub(crate) new_states: Vec<EvidenceState>,
}

impl CallEvidenceCtx {
    pub fn new(current_states: VecDeque<EvidenceState>) -> Self {
        let right = current_states.len();
        Self {
            current_states,
            used_states_in_subtree: 0,
            subtree_size: right,
            new_states: vec![],
        }
    }
}
