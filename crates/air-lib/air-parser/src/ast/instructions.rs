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

mod impls;
mod traits;

use super::*;
use serde::Serialize;
use std::rc::Rc;

// TODO: sort instruction in alphanumeric order
#[allow(clippy::large_enum_variant)] // for Null and Error variants
#[derive(Serialize, Debug, PartialEq)]
pub enum Instruction<'i> {
    Call(Call<'i>),
    Ap(Ap<'i>),
    Seq(Seq<'i>),
    Par(Par<'i>),
    Xor(Xor<'i>),
    Match(Match<'i>),
    MisMatch(MisMatch<'i>),
    Fail(Fail<'i>),
    FoldScalar(FoldScalar<'i>),
    FoldStream(FoldStream<'i>),
    New(New<'i>),
    Next(Next<'i>),
    Null(Null),
    Error,
}

/// (call (peer part of a triplet: PeerPart) (function part of a triplet: FunctionPart) [arguments] output)
#[derive(Serialize, Debug, PartialEq)]
pub struct Call<'i> {
    pub triplet: Triplet<'i>,
    pub args: Rc<Vec<Value<'i>>>,
    pub output: CallOutputValue<'i>,
}

/// (ap argument result)
#[derive(Serialize, Debug, PartialEq)]
pub struct Ap<'i> {
    pub argument: ApArgument<'i>,
    pub result: Variable<'i>,
}

/// (seq instruction instruction)
#[derive(Serialize, Debug, PartialEq)]
pub struct Seq<'i>(pub Box<Instruction<'i>>, pub Box<Instruction<'i>>);

/// (par instruction instruction)
#[derive(Serialize, Debug, PartialEq)]
pub struct Par<'i>(pub Box<Instruction<'i>>, pub Box<Instruction<'i>>);

/// (xor instruction instruction)
#[derive(Serialize, Debug, PartialEq)]
pub struct Xor<'i>(pub Box<Instruction<'i>>, pub Box<Instruction<'i>>);

/// (match left_value right_value instruction)
#[derive(Serialize, Debug, PartialEq)]
pub struct Match<'i> {
    pub left_value: Value<'i>,
    pub right_value: Value<'i>,
    pub instruction: Box<Instruction<'i>>,
}

/// (mismatch left_value right_value instruction)
#[derive(Serialize, Debug, PartialEq)]
pub struct MisMatch<'i> {
    pub left_value: Value<'i>,
    pub right_value: Value<'i>,
    pub instruction: Box<Instruction<'i>>,
}

/// (fold scalar_iterable iterator instruction)
#[derive(Serialize, Debug, PartialEq)]
pub enum Fail<'i> {
    Literal {
        ret_code: i64,
        error_message: &'i str,
    },
    LastError,
}

/// (fold scalar_iterable iterator instruction)
#[derive(Serialize, Debug, PartialEq)]
pub struct FoldScalar<'i> {
    pub iterable: ScalarWithLambda<'i>,
    #[serde(borrow)]
    pub iterator: Scalar<'i>,
    pub instruction: Rc<Instruction<'i>>,
    pub span: Span,
}

/// (fold stream_iterable iterator instruction)
#[derive(Serialize, Debug, PartialEq)]
pub struct FoldStream<'i> {
    pub iterable: Stream<'i>,
    #[serde(borrow)]
    pub iterator: Scalar<'i>,
    pub instruction: Rc<Instruction<'i>>,
    pub span: Span,
}

/// (fold stream_iterable iterator instruction)
#[derive(Serialize, Debug, PartialEq)]
pub struct Next<'i> {
    pub iterator: Scalar<'i>,
}

/// (new variable instruction)
#[derive(Serialize, Debug, PartialEq)]
pub struct New<'i> {
    pub variable: Variable<'i>,
    pub instruction: Box<Instruction<'i>>,
    pub span: Span,
}

/// (null)
#[derive(Serialize, Debug, PartialEq)]
pub struct Null;
