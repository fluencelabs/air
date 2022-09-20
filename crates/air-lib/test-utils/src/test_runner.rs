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
use crate::native_test_runner::NativeAirRunner as AirRunnerImpl;
#[cfg(not(feature = "test_with_native_code"))]
use crate::wasm_test_runner::WasmAirRunner as AirRunnerImpl;

use super::CallServiceClosure;
use avm_server::avm_runner::*;

use crate::print_trace;
use std::collections::HashMap;
use std::collections::HashSet;

pub trait AirRunner {
    fn new(current_call_id: impl Into<String>) -> Self;

    #[allow(clippy::too_many_arguments)]
    fn call(
        &mut self,
        air: impl Into<String>,
        prev_data: impl Into<Vec<u8>>,
        data: impl Into<Vec<u8>>,
        init_peer_id: impl Into<String>,
        timestamp: u64,
        ttl: u32,
        call_results: avm_server::CallResults,
    ) -> Result<RawAVMOutcome, Box<dyn std::error::Error>>;
}

pub struct TestRunner<R = AirRunnerImpl> {
    pub runner: R,
    pub call_service: CallServiceClosure,
}

#[derive(Debug, Default, Clone)]
pub struct TestRunParameters {
    pub init_peer_id: String,
    pub timestamp: u64,
    pub ttl: u32,
}

impl<R: AirRunner> TestRunner<R> {
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
            println!("call_results: {:?}", call_results);
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
            print_trace(&outcome, "");

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

pub fn create_avm(
    call_service: CallServiceClosure,
    current_peer_id: impl Into<String>,
) -> TestRunner {
    let runner = AirRunnerImpl::new(current_peer_id);

    TestRunner {
        runner,
        call_service,
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
