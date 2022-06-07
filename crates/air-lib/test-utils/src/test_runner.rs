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

#[cfg(feature = "test_with_native_code")]
pub use crate::native_test_runner::{create_avm, TestRunner};
#[cfg(not(feature = "test_with_native_code"))]
pub use crate::wasm_test_runner::{create_avm, TestRunner};

use avm_server::avm_runner::*;

use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Debug, Default, Clone)]
pub struct TestRunParameters {
    pub init_peer_id: String,
    pub timestamp: u64,
    pub ttl: u32,
}

impl TestRunner {
    pub fn call(
        &mut self,
        air: impl Into<String>,
        prev_data: impl Into<Vec<u8>>,
        data: impl Into<Vec<u8>>,
        test_run_params: TestRunParameters,
    ) -> Result<RawAVMOutcome, String> {
        let air = air.into();
        let mut prev_data = prev_data.into();
        let mut data = data.into();

        let TestRunParameters {
            init_peer_id,
            timestamp,
            ttl,
        } = test_run_params;

        let mut call_results = HashMap::new();
        let mut next_peer_pks = HashSet::new();

        loop {
            let mut outcome: RawAVMOutcome = self
                .runner
                .call(
                    air.clone(),
                    prev_data,
                    data,
                    init_peer_id.clone(),
                    timestamp,
                    ttl,
                    call_results,
                )
                .map_err(|e| e.to_string())?;

            next_peer_pks.extend(outcome.next_peer_pks);

            if outcome.call_requests.is_empty() {
                outcome.next_peer_pks = next_peer_pks.into_iter().collect::<Vec<_>>();
                return Ok(outcome);
            }

            call_results = outcome
                .call_requests
                .into_iter()
                .map(|(id, call_parameters)| {
                    let service_result = (self.call_service)(call_parameters);
                    (id, service_result)
                })
                .collect::<HashMap<_, _>>();

            prev_data = outcome.data;
            data = vec![];
        }
    }
}

impl TestRunParameters {
    pub fn new(init_peer_id: impl Into<String>, timestamp: u64, ttl: u32) -> Self {
        Self {
            init_peer_id: init_peer_id.into(),
            timestamp,
            ttl,
        }
    }

    pub fn from_init_peer_id(init_peer_id: impl Into<String>) -> Self {
        Self {
            init_peer_id: init_peer_id.into(),
            timestamp: 0,
            ttl: 0,
        }
    }

    pub fn from_timestamp(timestamp: u64) -> Self {
        Self {
            init_peer_id: String::new(),
            timestamp,
            ttl: 0,
        }
    }

    pub fn from_ttl(ttl: u32) -> Self {
        Self {
            init_peer_id: String::new(),
            timestamp: 0,
            ttl,
        }
    }
}
