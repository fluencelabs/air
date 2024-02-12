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

use super::{FunctionOutcome, MarineService};
use crate::asserts::ServiceDefinition;

use futures::future::LocalBoxFuture;
use futures::FutureExt;

use air_test_utils::CallRequestParams;

use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ResultStore {
    pub(crate) results: RefCell<HashMap<usize, ServiceDefinition>>,
}

impl ResultStore {
    pub(crate) fn insert(&self, service_definition: ServiceDefinition) -> Result<usize, String> {
        let mut results = self.results.borrow_mut();
        let id = results.len();
        results.insert(id, service_definition);
        Ok(id)
    }
}

impl MarineService for ResultStore {
    fn call<'this>(
        &'this self,
        mut params: CallRequestParams,
    ) -> LocalBoxFuture<'this, FunctionOutcome> {
        async {
            let results = self.results.borrow();
            let (real_service_id, suffix) = match params.service_id.rsplit_once("..") {
                Some(split) => split,
                None => return FunctionOutcome::NotDefined,
            };

            if let Ok(result_id) = suffix.parse::<usize>() {
                let service_desc = results.get(&result_id).unwrap_or_else(|| {
                    panic!("failed to parse service name {:?}", params.service_id)
                });
                // hide the artificial service_id
                params.service_id = real_service_id.to_owned();
                FunctionOutcome::from_service_result(service_desc.call(params).await)
            } else {
                // Pass malformed service names further in a chain
                FunctionOutcome::NotDefined
            }
        }
        .boxed_local()
    }
}

pub(crate) struct MarineServiceWrapper<T> {
    wrapped: Rc<T>,
}

impl<T> MarineServiceWrapper<T> {
    pub(crate) fn new(wrapped: Rc<T>) -> Self {
        Self { wrapped }
    }
}

impl<T: MarineService> MarineService for MarineServiceWrapper<T> {
    fn call<'this>(
        &'this self,
        params: CallRequestParams,
    ) -> LocalBoxFuture<'this, FunctionOutcome> {
        self.wrapped.call(params)
    }
}
