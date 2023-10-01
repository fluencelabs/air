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

#![forbid(unsafe_code)]
#![warn(rust_2018_idioms)]
#![deny(
    dead_code,
    nonstandard_style,
    unused_imports,
    unused_mut,
    unused_variables,
    unused_unsafe,
    unreachable_patterns
)]

use serde::Deserialize;
use serde::Serialize;

use std::fmt;
use std::marker::PhantomData;

#[derive(Serialize, Deserialize)]
#[cfg_attr(
    feature = "borsh",
    derive(::borsh::BorshSerialize, ::borsh::BorshDeserialize)
)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[cfg_attr(feature = "rkyv", archive(check_bytes, compare(PartialEq, PartialOrd)))]
#[cfg_attr(feature = "rkyv", omit_bounds)] // TODO look close, it can be a misuse
#[serde(transparent)]
pub struct CID<T: ?Sized>(
    String,
    #[serde(skip)]
    #[cfg_attr(feature = "borsh", borsh_skip)]
    PhantomData<*const T>,
);

impl<T: ?Sized> CID<T> {
    fn from_cid(cid: cid::Cid) -> Self {
        Self(cid.to_string(), PhantomData)
    }

    pub fn into_inner(self) -> String {
        self.0
    }

    pub fn new(cid_str: &str) -> Self {
        Self(cid_str.to_owned(), PhantomData)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<T: ?Sized> Clone for CID<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1)
    }
}

impl<T: ?Sized> fmt::Debug for CID<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("CID").field(&self.0).finish()
    }
}

impl<T: ?Sized> std::fmt::Display for CID<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<Val> PartialEq for CID<Val> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<Val> Ord for CID<Val> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        Ord::cmp(&self.0, &other.0)
    }
}

impl<Val> PartialOrd for CID<Val> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(Ord::cmp(self, other))
    }
}

impl<Val> Eq for CID<Val> {}

impl<Val> std::hash::Hash for CID<Val> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        self.1.hash(state);
    }
}

impl<T: ?Sized> From<CID<T>> for Vec<u8> {
    fn from(value: CID<T>) -> Self {
        value.0.into()
    }
}

#[cfg(feature = "rkyv")]
impl<T: ?Sized> PartialEq for ArchivedCID<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_str() == other.0.as_str()
    }
}

#[cfg(feature = "rkyv")]
impl<T: ?Sized> Eq for ArchivedCID<T> {}

#[cfg(feature = "rkyv")]
impl<Val> std::hash::Hash for ArchivedCID<Val> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        // self.1.hash(state);
    }
}

#[cfg(feature = "rkyv")]
impl<Val> Ord for ArchivedCID<Val> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        Ord::cmp(self.0.as_str(), other.0.as_str())
    }
}

#[cfg(feature = "rkyv")]
impl<Val> PartialOrd for ArchivedCID<Val> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(Ord::cmp(self, other))
    }
}

// TODO we might refactor this to `SerializationFormat` trait
// that both transform data to binary/text form (be it JSON, CBOR or something else)
// and produces CID too
pub fn json_data_cid<Val: ?Sized>(data: &[u8]) -> CID<Val> {
    use cid::Cid;
    use multihash::{Code, MultihashDigest};

    // the Sha2_256 is current IPFS default hash
    let digest = Code::Sha2_256.digest(data);
    // seems to be better than RAW_CODEC = 0x55
    const JSON_CODEC: u64 = 0x0200;

    let cid = Cid::new_v1(JSON_CODEC, digest);
    CID::from_cid(cid)
}

pub struct CidCalculationError(serde_json::Error);

impl fmt::Debug for CidCalculationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl fmt::Display for CidCalculationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl From<serde_json::Error> for CidCalculationError {
    fn from(source: serde_json::Error) -> Self {
        Self(source)
    }
}

impl std::error::Error for CidCalculationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

/// Calculate a CID of JSON-serialized value.
pub fn value_to_json_cid<Val: Serialize>(value: &Val) -> Result<CID<Val>, CidCalculationError> {
    let data = serde_json::to_vec(value)?;
    Ok(json_data_cid(&data))
}
