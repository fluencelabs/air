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

use super::ExecutionCtx;
use super::ExecutionResult;
use super::TraceHandler;
use crate::execution_step::boxed_value::CanonStream;
use crate::execution_step::Generation;
use crate::log_instruction;
use crate::trace_to_exec_err;
use crate::CatchableError;
use crate::ExecutionError;
use crate::UncatchableError;

use air_interpreter_data::{CanonResult, TracePos};
use air_parser::ast;
use air_trace_handler::MergerCanonResult;
use std::rc::Rc;

impl<'i> super::ExecutableInstruction<'i> for ast::Canon<'i> {
    #[tracing::instrument(level = "debug", skip(exec_ctx, trace_ctx))]
    fn execute(&self, exec_ctx: &mut ExecutionCtx<'i>, trace_ctx: &mut TraceHandler) -> ExecutionResult<()> {
        log_instruction!(call, exec_ctx, trace_ctx);
        let canon_result = trace_to_exec_err!(trace_ctx.meet_canon_start(), self)?;

        match canon_result {
            MergerCanonResult::CanonResult { stream_elements_pos } => {
                handle_seen_canon(self, stream_elements_pos, exec_ctx, trace_ctx)
            }
            MergerCanonResult::Empty => handle_unseen_canon(self, exec_ctx, trace_ctx),
        }
    }
}

fn handle_seen_canon(
    ast_canon: &ast::Canon<'_>,
    stream_elements_pos: Vec<TracePos>,
    exec_ctx: &mut ExecutionCtx<'_>,
    trace_ctx: &mut TraceHandler,
) -> ExecutionResult<()> {
    let canon_stream = create_canon_stream_from_pos(&stream_elements_pos, &ast_canon.stream, exec_ctx)?;
    let stream_with_positions = StreamWithPositions {
        canon_stream,
        stream_elements_pos,
    };

    epilog(ast_canon.canon_stream.name, stream_with_positions, exec_ctx, trace_ctx);
    Ok(())
}

fn handle_unseen_canon(
    ast_canon: &ast::Canon<'_>,
    exec_ctx: &mut ExecutionCtx<'_>,
    trace_ctx: &mut TraceHandler,
) -> ExecutionResult<()> {
    let peer_id = crate::execution_step::air::resolve_to_string(&ast_canon.peer_pk, exec_ctx)?;

    if exec_ctx.run_parameters.current_peer_id.as_str() != peer_id {
        exec_ctx.subgraph_complete = false;
        exec_ctx.next_peer_pks.push(peer_id);
        return Ok(());
    }

    let stream_with_positions = create_canon_stream_from_name(&ast_canon.stream, exec_ctx)?;
    epilog(ast_canon.canon_stream.name, stream_with_positions, exec_ctx, trace_ctx);
    Ok(())
}

fn create_canon_stream_from_pos(
    stream_elements_pos: &[TracePos],
    stream: &ast::Stream<'_>,
    exec_ctx: &ExecutionCtx<'_>,
) -> ExecutionResult<CanonStream> {
    let stream = exec_ctx.streams.get(stream.name, stream.position).ok_or_else(|| {
        ExecutionError::Catchable(Rc::new(CatchableError::StreamsForCanonNotFound(
            stream.name.to_string(),
        )))
    })?;

    let values = stream_elements_pos
        .iter()
        .map(|&position| {
            stream
                .get_value_by_pos(position)
                .ok_or(ExecutionError::Uncatchable(UncatchableError::VariableNotFoundByPos(
                    position,
                )))
                .cloned()
        })
        .collect::<Result<Vec<_>, _>>()?;

    let canon_stream = CanonStream::new(values);
    Ok(canon_stream)
}

fn epilog(
    canon_stream_name: &str,
    stream_with_positions: StreamWithPositions,
    exec_ctx: &mut ExecutionCtx<'_>,
    trace_ctx: &mut TraceHandler,
) {
    let StreamWithPositions {
        canon_stream,
        stream_elements_pos,
    } = stream_with_positions;

    exec_ctx.streams.add_canon(canon_stream_name, canon_stream);
    trace_ctx.meet_canon_end(CanonResult::new(stream_elements_pos));
}

struct StreamWithPositions {
    canon_stream: CanonStream,
    stream_elements_pos: Vec<TracePos>,
}

fn create_canon_stream_from_name(
    stream: &ast::Stream<'_>,
    exec_ctx: &ExecutionCtx<'_>,
) -> ExecutionResult<StreamWithPositions> {
    let stream = exec_ctx.streams.get(stream.name, stream.position).ok_or_else(|| {
        ExecutionError::Catchable(Rc::new(CatchableError::StreamsForCanonNotFound(
            stream.name.to_string(),
        )))
    })?;
    let canon_stream = CanonStream::from_stream(stream);
    let stream_elements_pos = stream
        .iter(Generation::Last)
        .unwrap()
        .map(|value| value.trace_pos)
        .collect::<Vec<_>>();

    let result = StreamWithPositions {
        canon_stream,
        stream_elements_pos,
    };

    Ok(result)
}
