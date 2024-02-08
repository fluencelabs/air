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

use crate::JValue;
use crate::RawValue;

use air_interpreter_cid::raw_value_to_json_cid;
use air_interpreter_cid::value_to_json_cid;
use air_interpreter_cid::verify_raw_value;
use air_interpreter_cid::verify_value;
use air_interpreter_cid::CidCalculationError;
use air_interpreter_cid::CidRef;
use air_interpreter_cid::CidVerificationError;
use air_interpreter_cid::CID;
use serde::Deserialize;
use serde::Serialize;
use thiserror::Error as ThisError;

use std::{collections::HashMap, rc::Rc};

/// Stores CID to Value corresponance.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(transparent)]
#[derive(::rkyv::Archive, ::rkyv::Serialize, ::rkyv::Deserialize)]
#[archive(check_bytes)]
pub struct CidStore<Val>(#[with(::rkyv::with::AsVec)] HashMap<CID<Val>, Rc<Val>>);

impl<Val> CidStore<Val> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, cid: &CID<Val>) -> Option<Rc<Val>> {
        self.0.get(cid).cloned()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&CID<Val>, &Rc<Val>)> {
        self.0.iter()
    }

    pub fn check_reference<Src>(
        &self,
        _source_cid: &CID<Src>,
        target_cid: &CID<Val>,
    ) -> Result<(), CidStoreVerificationError> {
        self.0
            .get(target_cid)
            .ok_or_else(|| CidStoreVerificationError::MissingReference {
                source_type_name: std::any::type_name::<Src>(),
                target_type_name: std::any::type_name::<Val>(),
                target_cid_repr: target_cid.get_inner(),
            })?;
        Ok(())
    }
}

impl<Val: Serialize> CidStore<Val> {
    pub fn verify(&self) -> Result<(), CidStoreVerificationError> {
        for (cid, value) in &self.0 {
            verify_value(cid, value)?;
        }
        Ok(())
    }
}

impl CidStore<RawValue> {
    pub fn verify_raw_value(&self) -> Result<(), CidStoreVerificationError> {
        for (cid, value) in &self.0 {
            verify_raw_value(cid, value.as_inner())?;
        }
        Ok(())
    }
}

#[derive(ThisError, Debug)]
pub enum CidStoreVerificationError {
    #[error(transparent)]
    CidVerificationError(#[from] CidVerificationError),

    #[error("Reference CID {target_cid_repr:?} from type {source_type_name:?} to {target_type_name:?} was not found")]
    MissingReference {
        source_type_name: &'static str,
        target_type_name: &'static str,
        target_cid_repr: Rc<CidRef>,
    },
}

impl<Val> Default for CidStore<Val> {
    fn default() -> Self {
        Self(Default::default())
    }
}

#[derive(Clone, Debug)]
pub struct CidTracker<Val = JValue> {
    cids: HashMap<CID<Val>, Rc<Val>>,
}

impl<Val> CidTracker<Val> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_cid_stores(prev_cid_map: CidStore<Val>, current_cid_map: CidStore<Val>) -> Self {
        let mut cids = prev_cid_map.0;
        for (cid, val) in current_cid_map.0 {
            // TODO check that values matches?
            cids.insert(cid, val);
        }
        Self { cids }
    }

    pub fn get(&self, cid: &CID<Val>) -> Option<Rc<Val>> {
        self.cids.get(cid).cloned()
    }
}

impl<Val: Serialize> CidTracker<Val> {
    pub fn track_value(
        &mut self,
        value: impl Into<Rc<Val>>,
    ) -> Result<CID<Val>, CidCalculationError> {
        let value = value.into();
        let cid = value_to_json_cid(&*value)?;
        self.cids.insert(cid.clone(), value);
        Ok(cid)
    }
}

impl CidTracker<RawValue> {
    pub fn track_raw_value(&mut self, value: impl Into<Rc<RawValue>>) -> CID<RawValue> {
        let value = value.into();
        let cid = raw_value_to_json_cid(value.as_inner());
        self.cids.insert(cid.clone(), value);
        cid
    }
}

impl<Val> Default for CidTracker<Val> {
    fn default() -> Self {
        Self {
            cids: Default::default(),
        }
    }
}

impl<Val> From<CidTracker<Val>> for CidStore<Val> {
    fn from(value: CidTracker<Val>) -> Self {
        Self(value.cids)
    }
}

impl<Val> IntoIterator for CidStore<Val> {
    type Item = (CID<Val>, Rc<Val>);

    type IntoIter = std::collections::hash_map::IntoIter<CID<Val>, Rc<Val>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use std::iter::FromIterator;

    use super::*;
    use serde_json::json;

    #[tokio::test]
    fn test_iter() {
        let mut tracker = CidTracker::new();
        tracker.track_value(json!("test")).unwrap();
        tracker.track_value(json!(1)).unwrap();
        tracker.track_value(json!([1, 2, 3])).unwrap();
        tracker
            .track_value(json!({
                "key": 42,
            }))
            .unwrap();
        let store = CidStore::from(tracker);
        assert_eq!(
            store.into_iter().collect::<HashMap<_, _>>(),
            HashMap::from_iter(vec![
                (
                    CID::new("bagaaihrarcyykpv4oj7zwdbepczyfthxya4og7s2rwvrzolm5kg2eu5dz3xa")
                        .into(),
                    json!("test").into()
                ),
                (
                    CID::new("bagaaihram6sitn77tquub77n2jzjgttrlwkverv44pv3gns6qghm6hx6d36a")
                        .into(),
                    json!([1, 2, 3]).into(),
                ),
                (
                    CID::new("bagaaihra2y55tkbgv6i4d7vdoglfuzhbd3ra6e7ennpvfrmzaejwmbntusdq")
                        .into(),
                    json!(1).into(),
                ),
                (
                    CID::new("bagaaihracpzxhsrpviexa7k6glwdhyh3a4kvy6j7qlcqokzqbs3q424cmxyq")
                        .into(),
                    json!({
                        "key": 42,
                    })
                    .into(),
                )
            ])
        );
    }

    #[tokio::test]
    fn test_store() {
        let mut tracker = CidTracker::new();
        tracker.track_value(json!("test")).unwrap();
        tracker.track_value(json!(1)).unwrap();
        tracker.track_value(json!([1, 2, 3])).unwrap();
        tracker
            .track_value(json!({
                "key": 42,
            }))
            .unwrap();
        let store = CidStore::from(tracker);

        assert_eq!(
            &*store
                .get(&CID::new(
                    "bagaaihrarcyykpv4oj7zwdbepczyfthxya4og7s2rwvrzolm5kg2eu5dz3xa"
                ))
                .unwrap(),
            &json!("test"),
        );
        assert_eq!(
            &*store
                .get(&CID::new(
                    "bagaaihram6sitn77tquub77n2jzjgttrlwkverv44pv3gns6qghm6hx6d36a"
                ))
                .unwrap(),
            &json!([1, 2, 3]),
        );
        assert_eq!(
            &*store
                .get(&CID::new(
                    "bagaaihra2y55tkbgv6i4d7vdoglfuzhbd3ra6e7ennpvfrmzaejwmbntusdq"
                ))
                .unwrap(),
            &json!(1),
        );
        assert_eq!(
            &*store
                .get(&CID::new(
                    "bagaaihracpzxhsrpviexa7k6glwdhyh3a4kvy6j7qlcqokzqbs3q424cmxyq"
                ))
                .unwrap(),
            &json!({"key": 42}),
        );

        assert_eq!(store.get(&CID::new("loremimpsumdolorsitament")), None);
    }
}
