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

use air_log_targets::TARGET_MAP;

use log::LevelFilter;

pub fn init_logger(default_level: Option<LevelFilter>) {
    let target_map = TARGET_MAP.iter().cloned().collect();
    let builder = marine_rs_sdk::WasmLoggerBuilder::new()
        .with_target_map(target_map)
        .filter("jsonpath_lib", log::LevelFilter::Info);

    let builder = if let Some(default_level) = default_level {
        builder.with_log_level(default_level)
    } else {
        builder
    };

    builder.build().unwrap();
}

// this the only variable allowed to access by Marine WASI configuration,
// and it can be used in log-compatible fashion
pub const AQUAVM_TRACING_ENV: &str = "WASM_LOG";

// TODO it worth moving it to marine_rs_sdk
pub fn init_tracing() {
    use tracing_subscriber::fmt::format::FmtSpan;
    use tracing_subscriber::EnvFilter;

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_env(AQUAVM_TRACING_ENV))
        .json() // remove this line for nice human-readable output
        .with_span_events(FmtSpan::ENTER | FmtSpan::CLOSE)
        .init();
}
