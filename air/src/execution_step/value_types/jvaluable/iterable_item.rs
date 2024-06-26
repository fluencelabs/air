/*
 * AquaVM Workflow Engine
 *
 * Copyright (C) 2024 Fluence DAO
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation version 3 of the
 * License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use super::select_by_lambda_from_scalar;
use super::ExecutionResult;
use super::IterableItem;
use super::JValuable;
use super::LambdaAST;
use crate::execution_step::value_types::populate_tetraplet_with_lambda;
use crate::execution_step::ExecutionCtx;
use crate::execution_step::RcSecurityTetraplets;
use crate::JValue;
use crate::SecurityTetraplet;

use air_interpreter_data::Provenance;

impl<'ctx> JValuable for IterableItem<'ctx> {
    fn apply_lambda(&self, lambda: &LambdaAST<'_>, exec_ctx: &ExecutionCtx<'_>) -> ExecutionResult<JValue> {
        use super::IterableItem::*;

        let jvalue = match self {
            RefValue((jvalue, ..)) => jvalue,
            RcValue((jvalue, ..)) => jvalue,
        };

        let selected_value = select_by_lambda_from_scalar(jvalue, lambda, exec_ctx)?;
        Ok(selected_value)
    }

    fn apply_lambda_with_tetraplets(
        &self,
        lambda: &LambdaAST<'_>,
        exec_ctx: &ExecutionCtx<'_>,
        _root_provenance: &Provenance,
    ) -> ExecutionResult<(JValue, SecurityTetraplet, Provenance)> {
        use super::IterableItem::*;

        let (jvalue, tetraplet, provenance) = match self {
            RefValue((jvalue, tetraplet, _, provenance)) => (*jvalue, tetraplet, provenance),
            RcValue((jvalue, tetraplet, _, provenance)) => (jvalue, tetraplet, provenance),
        };

        let selected_value = select_by_lambda_from_scalar(jvalue, lambda, exec_ctx)?;
        let tetraplet = populate_tetraplet_with_lambda(tetraplet.as_ref().clone(), lambda);

        Ok((selected_value, tetraplet, provenance.clone()))
    }

    #[inline]
    fn as_jvalue(&self) -> JValue {
        use super::IterableItem::*;

        match self {
            RefValue((jvalue, ..)) => (*jvalue).clone(),
            RcValue((jvalue, ..)) => jvalue.clone(),
        }
    }

    fn as_tetraplets(&self) -> RcSecurityTetraplets {
        use super::IterableItem::*;

        // these clones are needed because rust-sdk allows passing arguments only by value
        match self {
            RefValue((_, tetraplet, _, _)) => vec![tetraplet.clone()],
            RcValue((_, tetraplet, _, _)) => vec![tetraplet.clone()],
        }
    }
}
