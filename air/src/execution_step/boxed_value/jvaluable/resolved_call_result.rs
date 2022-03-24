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

use super::select_from_scalar;
use super::ExecutionResult;
use super::JValuable;
use super::AIRLambdaAST;
use super::ValueAggregate;
use crate::execution_step::ExecutionCtx;
use crate::execution_step::RcSecurityTetraplets;
use crate::JValue;
use crate::SecurityTetraplet;

use air_lambda_ast::format_ast;

use std::borrow::Cow;
use std::ops::Deref;

impl JValuable for ValueAggregate {
    fn apply_lambda<'i>(&self, lambda: &AIRLambdaAST<'_>, exec_ctx: &ExecutionCtx<'i>) -> ExecutionResult<&JValue> {
        let selected_value = select_from_scalar(&self.result, lambda.iter(), exec_ctx)?;
        Ok(selected_value)
    }

    fn apply_lambda_with_tetraplets<'i>(
        &self,
        lambda: &AIRLambdaAST<'_>,
        exec_ctx: &ExecutionCtx<'i>,
    ) -> ExecutionResult<(&JValue, SecurityTetraplet)> {
        let selected_value = select_from_scalar(&self.result, lambda.iter(), exec_ctx)?;
        let mut tetraplet = self.tetraplet.as_ref().clone();
        tetraplet.add_lambda(&format_ast(lambda));

        Ok((selected_value, tetraplet))
    }

    fn as_jvalue(&self) -> Cow<'_, JValue> {
        Cow::Borrowed(&self.result)
    }

    fn into_jvalue(self: Box<Self>) -> JValue {
        self.result.deref().clone()
    }

    fn as_tetraplets(&self) -> RcSecurityTetraplets {
        vec![self.tetraplet.clone()]
    }
}
