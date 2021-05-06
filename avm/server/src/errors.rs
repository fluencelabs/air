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

use fluence_faas::FaaSError;

use std::io::Error as IOError;
use std::error::Error;
use std::path::PathBuf;

#[derive(Debug)]
pub enum AquamarineVMError {
    /// FaaS errors.
    FaaSError(FaaSError),

    /// Aquamarine stepper result deserialization errors.
    StepperResultDeError(String),

    /// I/O errors while persisting resulted data.
    PersistDataError(IOError, PathBuf),

    /// Errors related to particle_data_store path from supplied config.
    InvalidDataStorePath(IOError, PathBuf),

    /// Specified path to AIR interpreter .wasm file was invalid
    InvalidAquamarinePath {
        invalid_path: PathBuf,
        io_error: Option<IOError>,
        reason: &'static str,
    },
}

impl Error for AquamarineVMError {}

impl std::fmt::Display for AquamarineVMError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            AquamarineVMError::FaaSError(err) => write!(f, "{}", err),
            AquamarineVMError::StepperResultDeError(err_msg) => write!(f, "{}", err_msg),
            AquamarineVMError::PersistDataError(err, path) => write!(
                f,
                "an error occurred while saving prev data {:?} by {:?} path",
                err, path
            ),
            AquamarineVMError::InvalidDataStorePath(err, path) => write!(
                f,
                "an error occurred while creating data storage {:?} by {:?} path",
                err, path
            ),

            AquamarineVMError::InvalidAquamarinePath {
                invalid_path,
                io_error,
                reason,
            } => write!(
                f,
                "path to AIR interpreter .wasm ({:?}) is invalid: {}; IO Error: {:?}",
                invalid_path, reason, io_error
            ),
        }
    }
}

impl From<FaaSError> for AquamarineVMError {
    fn from(err: FaaSError) -> Self {
        AquamarineVMError::FaaSError(err)
    }
}

impl From<std::convert::Infallible> for AquamarineVMError {
    fn from(_: std::convert::Infallible) -> Self {
        unreachable!()
    }
}
