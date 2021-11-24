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

use super::*;

impl<'i> Ap<'i> {
    pub fn new(argument: ApArgument<'i>, result: Variable<'i>) -> Self {
        Self { argument, result }
    }
}

impl<'i> Call<'i> {
    pub fn new(
        triplet: Triplet<'i>,
        args: Rc<Vec<Value<'i>>>,
        output: CallOutputValue<'i>,
    ) -> Self {
        Self {
            triplet,
            args,
            output,
        }
    }
}

impl<'i> Seq<'i> {
    pub fn new(
        left_instruction: Box<Instruction<'i>>,
        right_instruction: Box<Instruction<'i>>,
    ) -> Self {
        Self(left_instruction, right_instruction)
    }
}

impl<'i> Par<'i> {
    pub fn new(
        left_instruction: Box<Instruction<'i>>,
        right_instruction: Box<Instruction<'i>>,
    ) -> Self {
        Self(left_instruction, right_instruction)
    }
}

impl<'i> Xor<'i> {
    pub fn new(
        left_instruction: Box<Instruction<'i>>,
        right_instruction: Box<Instruction<'i>>,
    ) -> Self {
        Self(left_instruction, right_instruction)
    }
}

impl<'i> Match<'i> {
    pub fn new(
        left_value: Value<'i>,
        right_value: Value<'i>,
        instruction: Box<Instruction<'i>>,
    ) -> Self {
        Self {
            left_value,
            right_value,
            instruction,
        }
    }
}

impl<'i> MisMatch<'i> {
    pub fn new(
        left_value: Value<'i>,
        right_value: Value<'i>,
        instruction: Box<Instruction<'i>>,
    ) -> Self {
        Self {
            left_value,
            right_value,
            instruction,
        }
    }
}

impl<'i> FoldScalar<'i> {
    pub fn new(
        iterable: ScalarWithLambda<'i>,
        iterator: Scalar<'i>,
        instruction: Instruction<'i>,
        span: Span,
    ) -> Self {
        Self {
            iterable,
            iterator,
            instruction: Rc::new(instruction),
            span,
        }
    }
}

impl<'i> FoldStream<'i> {
    pub fn new(
        iterable: Stream<'i>,
        iterator: Scalar<'i>,
        instruction: Instruction<'i>,
        span: Span,
    ) -> Self {
        Self {
            iterable,
            iterator,
            instruction: Rc::new(instruction),
            span,
        }
    }
}

impl<'i> Next<'i> {
    pub fn new(iterator: Scalar<'i>) -> Self {
        Self { iterator }
    }
}

impl<'i> New<'i> {
    #[allow(clippy::self_named_constructors)]
    pub fn new(variable: Variable<'i>, instruction: Box<Instruction<'i>>, span: Span) -> Self {
        Self {
            variable,
            instruction,
            span,
        }
    }
}
