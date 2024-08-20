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

use air::{CatchableError, UncatchableError};
use air_interpreter_starlark::ExecutionError as StarlarkExecutionError;
use air_test_utils::prelude::*;

#[tokio::test]
async fn embed_basic() {
    let mut vm = create_avm(echo_call_service(), "").await;

    let script = r##"
        (seq
            (embed []
#"
"a string\nwith escape"
"#
                var)
            (call %init_peer_id% ("" "") [var] result_name))"##;

    let result = checked_call_vm!(vm, <_>::default(), script, "", "");
    assert!(result.next_peer_pks.is_empty());

    let expected_trace = vec![scalar!(
        json!("a string\nwith escape"),
        peer = "",
        args = ["a string\nwith escape"]
    )];

    let trace = trace_from_result(&result);
    assert_eq!(&*trace, expected_trace);
}

#[tokio::test]
async fn embed_args() {
    let init_peer_id = "my_id";
    let mut vm = create_avm(echo_call_service(), init_peer_id).await;

    let script = r##"
        (seq
           (call %init_peer_id% ("myservice" "myfunc") [42] arg)
           (seq
               (embed [arg]
#"
t = get_tetraplet(0)[0]
"{}: {}/{}:{}".format(get_value(0), t.peer_pk, t.service_id, t.function_name)
"#
                      var)
               (call %init_peer_id% ("" "") [var] result_name)))"##;

    let run_params = TestRunParameters::from_init_peer_id(init_peer_id);
    let result = checked_call_vm!(vm, run_params, script, "", "");
    assert!(result.next_peer_pks.is_empty());

    let expected_val = format!("42: {init_peer_id}/myservice:myfunc");
    let expected_trace = vec![
        scalar!(
            json!(42),
            peer = init_peer_id,
            service = "myservice",
            function = "myfunc",
            args = [42]
        ),
        scalar!(json!(expected_val), peer = init_peer_id, args = [expected_val]),
    ];

    let trace = trace_from_result(&result);
    assert_eq!(&*trace, expected_trace);
}

#[tokio::test]
async fn embed_error_fail() {
    let mut vm = create_avm(echo_call_service(), "").await;

    let script = r##"
        (xor
            (embed []
#"
fail(42, "my message")
"#
                var)
            (call %init_peer_id% ("" "") [%last_error%.$.error_code %last_error%.$.message] result_name))"##;

    let result = checked_call_vm!(vm, <_>::default(), script, "", "");
    assert!(result.next_peer_pks.is_empty());

    let expected_trace = vec![scalar!(json!(42), peer = "", args = [json!(42), json!("my message")])];

    let trace = trace_from_result(&result);
    assert_eq!(&*trace, expected_trace);
}

#[tokio::test]
async fn embed_error_value() {
    let mut vm = create_avm(echo_call_service(), "").await;

    let script = r##"
       (embed []
#"
42 + "string"
"#
              var)"##;

    let result = call_vm!(vm, <_>::default(), script, "", "");
    let expected_error = CatchableError::StarlarkError(StarlarkExecutionError::Value(
        "error: Operation `+` not supported for types `int` and `string`\n --> dummy.star:2:1\n  |\n2 | 42 + \"string\"\n  | ^^^^^^^^^^^^^\n  |\n".to_owned(),
    ));
    assert_error_eq!(&result, expected_error);
}

// TODO 42.length gives Other, and it is a problem
#[tokio::test]
async fn embed_error_lexer() {
    let mut vm = create_avm(echo_call_service(), "").await;

    let script = r##"
       (embed []
#"
"an unterminated string
"#
              var)"##;

    let result = call_vm!(vm, <_>::default(), script, "", "");
    let expected_error = UncatchableError::StarlarkError(StarlarkExecutionError::Lexer(
        "Parse error: unfinished string literal".to_owned(),
    ));
    assert_error_eq!(&result, expected_error);
}

#[tokio::test]
async fn embed_with_join_behavior() {
    let mut vm = create_avm(echo_call_service(), "").await;

    let script = r##"
        (par
            (call "other_peer" ("" "") [] var)
            (seq
                (embed [var] #"var + var"# var2)
                (call %init_peer_id% ("" "") [var2])))"##;

    let result = call_vm!(vm, <_>::default(), script, "", "");

    assert_eq!(result.error_message, "");
    assert_eq!(result.ret_code, 0);

    let trace = trace_from_result(&result);
    assert_eq!(trace.len(), 2);
}

#[tokio::test]
async fn embed_zip_reverse() {
    let mut vm = create_avm(echo_call_service(), "").await;

    let script = r##"
        (seq
            (seq
               (seq
                   (ap 1 $stream)
                   (ap 2 $stream))
            (canon %init_peer_id% $stream #canon))
            (seq
                (embed [#canon #canon]
#"
def main():
    v1 = get_value(0)
    v2 = get_value(1)

    if get_tetraplet(0)[0].peer_pk != get_tetraplet(1)[0].peer_pk:
        fail(42, 'tetraplet peer_pk mismatch')

    return list(zip(v1, reversed(v2)))

main()
"#
                       var2)
                (call %init_peer_id% ("" "") [var2] var3)))"##;

    let result = call_vm!(vm, <_>::default(), script, "", "");

    assert_eq!(result.error_message, "", "{}", result.error_message);
    assert_eq!(result.ret_code, 0);

    let expected_trace = vec![
        ap(0),
        ap(0),
        canon(json!({
            "tetraplet":  {"function_name": "", "lens": "", "peer_pk": "", "service_id": ""},
            "values": [{
                "tetraplet": {"function_name": "", "lens": "", "peer_pk": "", "service_id": ""},
                "result": 1
            }, {
                "tetraplet": {"function_name": "", "lens": "", "peer_pk": "", "service_id": ""},
                "result": 2
            }]
        })),
        scalar!(json!([(1, 2), (2, 1)]), args = [json!([(1, 2), (2, 1)])]),
    ];
    let data = data_from_result(&result);
    let trace = trace_from_result(&result);
    assert_eq!(&*trace, expected_trace, "{:#?}", data.cid_info);
}
