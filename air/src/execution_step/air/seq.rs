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

use super::ExecutionCtx;
use super::ExecutionResult;
use super::TraceHandler;
use crate::log_instruction;

use air_parser::ast::Seq;

impl<'i> super::ExecutableInstruction<'i> for Seq<'i> {
    fn execute(&self, exec_ctx: &mut ExecutionCtx<'i>, trace_ctx: &mut TraceHandler) -> ExecutionResult<()> {
        log_instruction!(seq, exec_ctx, trace_ctx);

        exec_ctx.subgraph_complete = true;
        self.0.execute(exec_ctx, trace_ctx)?;
        // println!("> seq {}", exec_ctx.subgraph_complete);

        if exec_ctx.subgraph_complete {
            self.1.execute(exec_ctx, trace_ctx)?;
        }

        Ok(())
    }
}
