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

use std::borrow::Cow;

/// Represents parameters obtained from a particle.
pub struct ParticleParameters<'init_peer_id, 'particle_id> {
    pub init_peer_id: Cow<'init_peer_id, String>,
    pub particle_id: &'particle_id str,
    pub timestamp: u64,
}

impl<'init_peer_id, 'particle_id> ParticleParameters<'init_peer_id, 'particle_id> {
    pub fn new(
        init_peer_id: Cow<'init_peer_id, String>,
        particle_id: &'particle_id str,
        timestamp: u64,
    ) -> Self {
        Self {
            init_peer_id,
            particle_id,
            timestamp,
        }
    }
}
