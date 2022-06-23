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

use super::CallResults;
use crate::raw_outcome::RawAVMOutcome;
use crate::RunnerError;
use crate::RunnerResult;

use air_interpreter_interface::InterpreterOutcome;
use fluence_faas::FaaSConfig;
use fluence_faas::FluenceFaaS;
use fluence_faas::IValue;
use fluence_faas::ModuleDescriptor;

use std::path::PathBuf;

pub struct AVMRunner {
    faas: FluenceFaaS,
    current_peer_id: String,
    /// file name of the AIR interpreter .wasm
    wasm_filename: String,
}

/// Return statistic of AVM server Wasm module heap footprint.
pub struct AVMMemoryStats {
    /// Size of currently used linear memory in bytes.
    /// Please note that linear memory contains not only heap, but globals, shadow stack and so on.
    pub memory_size: usize,
    /// Possibly set max memory size for AVM server.
    pub max_memory_size: Option<usize>,
}

impl AVMRunner {
    /// Create AVM with the provided config.
    pub fn new(
        air_wasm_path: PathBuf,
        current_peer_id: impl Into<String>,
        max_heap_size: Option<u64>,
        logging_mask: i32,
    ) -> RunnerResult<Self> {
        let (wasm_dir, wasm_filename) = split_dirname(air_wasm_path)?;

        let faas_config = make_faas_config(wasm_dir, &wasm_filename, max_heap_size, logging_mask);
        let faas = FluenceFaaS::with_raw_config(faas_config)?;
        let current_peer_id = current_peer_id.into();

        let avm = Self {
            faas,
            current_peer_id,
            wasm_filename,
        };

        Ok(avm)
    }

    #[tracing::instrument(skip_all)]
    pub fn call(
        &mut self,
        air: impl Into<String>,
        prev_data: impl Into<Vec<u8>>,
        data: impl Into<Vec<u8>>,
        init_peer_id: impl Into<String>,
        timestamp: u64,
        ttl: u32,
        call_results: CallResults,
    ) -> RunnerResult<RawAVMOutcome> {
        let args = prepare_args(
            air,
            prev_data,
            data,
            self.current_peer_id.clone(),
            init_peer_id.into(),
            timestamp,
            ttl,
            call_results,
        );

        let result = {
            let span = tracing::span!(tracing::Level::INFO, "faas.call_with_ivalues", method="invoke");
            let _enter = span.enter();
            self.faas
                .call_with_ivalues(&self.wasm_filename, "invoke", &args, <_>::default())?
        };

        let result = try_as_one_value_vec(result)?;
        let outcome = InterpreterOutcome::from_ivalue(result)
            .map_err(RunnerError::InterpreterResultDeError)?;
        let outcome = RawAVMOutcome::from_interpreter_outcome(outcome)?;

        Ok(outcome)
    }

    pub fn memory_stats(&self) -> AVMMemoryStats {
        let stats = self.faas.module_memory_stats();

        // only the interpreters must be loaded in FaaS
        debug_assert!(stats.len() == 1);

        AVMMemoryStats {
            memory_size: stats[0].memory_size,
            max_memory_size: stats[0].max_memory_size,
        }
    }

    #[inline]
    pub fn set_peer_id(&mut self, current_peer_id: impl Into<String>) {
        self.current_peer_id = current_peer_id.into();
    }
}

#[tracing::instrument(skip(air, prev_data, data, call_results))]
fn prepare_args(
    air: impl Into<String>,
    prev_data: impl Into<Vec<u8>>,
    data: impl Into<Vec<u8>>,
    current_peer_id: String,
    init_peer_id: String,
    timestamp: u64,
    ttl: u32,
    call_results: CallResults,
) -> Vec<IValue> {
    let run_parameters = air_interpreter_interface::RunParameters::new(
        init_peer_id,
        current_peer_id,
        timestamp,
        ttl,
    )
    .into_ivalue();

    let call_results = crate::interface::into_raw_result(call_results);
    let call_results = {
        let span = tracing::span!(tracing::Level::INFO, "serde_json::to_vec call_results");
        let _enter = span.enter();
        serde_json::to_vec(&call_results).expect("the default serializer shouldn't fail")
    };

    vec![
        IValue::String(air.into()),
        IValue::ByteArray(prev_data.into()),
        IValue::ByteArray(data.into()),
        run_parameters,
        IValue::ByteArray(call_results),
    ]
}

/// Splits given path into its directory and file name
///
/// # Example
/// For path `/path/to/air_interpreter_server.wasm` result will be `Ok(PathBuf(/path/to), "air_interpreter_server.wasm")`
fn split_dirname(path: PathBuf) -> RunnerResult<(PathBuf, String)> {
    use RunnerError::InvalidAIRPath;

    let metadata = path.metadata().map_err(|err| InvalidAIRPath {
        invalid_path: path.clone(),
        reason: "failed to get file's metadata (doesn't exist or invalid permissions)",
        io_error: Some(err),
    })?;

    if !metadata.is_file() {
        return Err(InvalidAIRPath {
            invalid_path: path,
            reason: "is not a file",
            io_error: None,
        });
    }

    let file_name = path
        .file_name()
        .expect("checked to be a file, file name must be defined");
    let file_name = file_name.to_string_lossy().into_owned();

    let mut path = path;
    // drop file name from path
    path.pop();

    Ok((path, file_name))
}

fn make_faas_config(
    air_wasm_dir: PathBuf,
    air_wasm_file: &str,
    max_heap_size: Option<u64>,
    logging_mask: i32,
) -> FaaSConfig {
    let air_module_config = fluence_faas::FaaSModuleConfig {
        mem_pages_count: None,
        max_heap_size,
        logger_enabled: true,
        host_imports: <_>::default(),
        wasi: None,
        logging_mask,
    };

    FaaSConfig {
        modules_dir: Some(air_wasm_dir),
        modules_config: vec![ModuleDescriptor {
            file_name: String::from(air_wasm_file),
            import_name: String::from(air_wasm_file),
            config: air_module_config,
        }],
        default_modules_config: None,
    }
}

fn try_as_one_value_vec(mut ivalues: Vec<IValue>) -> RunnerResult<IValue> {
    use RunnerError::IncorrectInterpreterResult;

    if ivalues.len() != 1 {
        return Err(IncorrectInterpreterResult(ivalues));
    }

    Ok(ivalues.remove(0))
}
