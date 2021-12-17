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

use air::ExecutionError;
use air_test_utils::prelude::*;

#[test]
fn match_equal() {
    let set_variable_peer_id = "set_variable_peer_id";
    let mut set_variable_vm = create_avm(echo_call_service(), set_variable_peer_id);

    let local_peer_id = "local_peer_id";
    let mut vm = create_avm(echo_call_service(), local_peer_id);

    let script = format!(
        r#"
            (seq
                (seq
                    (call "{0}" ("" "") ["value_1"] value_1)
                    (call "{0}" ("" "") ["value_1"] value_2)
                )
                (xor
                    (match value_1 value_2
                        (call "{1}" ("service_id_2" "local_fn_name") ["result_1"] result_1)
                    )
                    (call "{1}" ("service_id_2" "local_fn_name") ["result_2"] result_2)
                )
            )"#,
        set_variable_peer_id, local_peer_id
    );

    let result = checked_call_vm!(set_variable_vm, "asd", &script, "", "");
    let result = checked_call_vm!(vm, "asd", script, "", result.data);

    let actual_trace = trace_from_result(&result);
    let expected_state = executed_state::scalar_string("result_1");

    assert_eq!(actual_trace.len(), 3);
    assert_eq!(actual_trace[2], expected_state);
}

#[test]
fn match_not_equal() {
    let set_variable_peer_id = "set_variable_peer_id";
    let mut set_variable_vm = create_avm(echo_call_service(), set_variable_peer_id);

    let local_peer_id = "local_peer_id";
    let mut vm = create_avm(echo_call_service(), local_peer_id);

    let script = format!(
        r#"
            (seq
                (seq
                    (call "{0}" ("" "") ["value_1"] value_1)
                    (call "{0}" ("" "") ["value_2"] value_2)
                )
                (xor
                    (match value_1 value_2
                        (call "{1}" ("service_id_2" "local_fn_name") ["result_1"] result_1)
                    )
                    (call "{1}" ("service_id_2" "local_fn_name") ["result_2"] result_2)
                )
            )"#,
        set_variable_peer_id, local_peer_id
    );

    let result = checked_call_vm!(set_variable_vm, "asd", &script, "", "");
    let result = checked_call_vm!(vm, "asd", script, "", result.data);

    let actual_trace = trace_from_result(&result);
    let expected_state = executed_state::scalar_string("result_2");

    assert_eq!(actual_trace.len(), 3);
    assert_eq!(actual_trace[2], expected_state);
}

#[test]
fn match_with_string() {
    let set_variable_peer_id = "set_variable_peer_id";
    let mut set_variable_vm = create_avm(echo_call_service(), set_variable_peer_id);

    let local_peer_id = "local_peer_id";
    let mut vm = create_avm(echo_call_service(), local_peer_id);

    let script = format!(
        r#"
            (seq
                (call "{0}" ("" "") ["value_1"] value_1)
                (xor
                    (match value_1 "value_1"
                        (call "{1}" ("service_id_2" "local_fn_name") ["result_1"] result_1)
                    )
                    (call "{1}" ("service_id_2" "local_fn_name") ["result_2"] result_2)
                )
            )"#,
        set_variable_peer_id, local_peer_id
    );

    let result = checked_call_vm!(set_variable_vm, "asd", &script, "", "");
    let result = checked_call_vm!(vm, "asd", script, "", result.data);

    let actual_trace = trace_from_result(&result);
    let expected_state = executed_state::scalar_string("result_1");

    assert_eq!(actual_trace.len(), 2);
    assert_eq!(actual_trace[1], expected_state);
}

#[test]
fn match_with_init_peer_id() {
    let set_variable_peer_id = "set_variable_peer_id";
    let mut set_variable_vm = create_avm(echo_call_service(), set_variable_peer_id);

    let local_peer_id = "local_peer_id";
    let mut vm = create_avm(echo_call_service(), local_peer_id);

    let script = format!(
        r#"
            (seq
                (call "{0}" ("" "") ["{1}"] value_1)
                (xor
                    (match value_1 %init_peer_id%
                        (call "{1}" ("service_id_2" "local_fn_name") ["result_1"] result_1)
                    )
                    (call "{1}" ("service_id_2" "local_fn_name") ["result_2"] result_2)
                )
            )"#,
        set_variable_peer_id, local_peer_id
    );

    let result = checked_call_vm!(set_variable_vm, local_peer_id, &script, "", "");
    let result = checked_call_vm!(vm, local_peer_id, script, "", result.data);

    let actual_trace = trace_from_result(&result);
    let expected_executed_call_result = executed_state::scalar_string("result_1");

    assert_eq!(actual_trace.len(), 2);
    assert_eq!(actual_trace[1], expected_executed_call_result);
}

#[test]
fn match_with_equal_numbers() {
    let local_peer_id = "local_peer_id";
    let mut vm = create_avm(echo_call_service(), local_peer_id);

    let script = "
            (xor
                (match 1 1
                    (null)
                )
                (null)
            )";

    let _result = checked_call_vm!(vm, "asd", script, "", "");
}

#[test]
fn match_without_xor() {
    let set_variable_peer_id = "set_variable_peer_id";
    let mut set_variable_vm = create_avm(echo_call_service(), set_variable_peer_id);

    let local_peer_id = "local_peer_id";
    let mut vm = create_avm(echo_call_service(), local_peer_id);

    let script = format!(
        r#"
            (seq
                (seq
                    (call "{0}" ("" "") ["value_1"] value_1)
                    (call "{0}" ("" "") ["value_2"] value_2)
                )
                (match value_1 value_2
                    (call "{1}" ("service_id_2" "local_fn_name") ["result_1"] result_1)
                )
            )"#,
        set_variable_peer_id, local_peer_id
    );

    let result = call_vm!(set_variable_vm, "", &script, "", "");
    let result = call_vm!(vm, "", &script, "", result.data);

    let expected_error = rc!(ExecutionError::MatchWithoutXorError);
    assert!(check_error(&result, expected_error));

    let result = call_vm!(vm, "", script, "", result.data);

    let expected_error = rc!(ExecutionError::MatchWithoutXorError);
    assert!(check_error(&result, expected_error));
}

#[test]
fn match_with_two_xors() {
    let local_peer_id = "local_peer_id";
    let mut vm = create_avm(set_variable_call_service(serde_json::json!(false)), local_peer_id);

    let local_peer_id_2 = "local_peer_id_2";

    let script = format!(
        r#"
            (xor
                (seq
                    (seq
                        (call "{0}" ("getDataSrv" "condition") [] condition)
                        (call "{0}" ("getDataSrv" "relay") [] relay)
                    )
                    (xor
                        (match condition true
                            (call "{0}" ("println" "print") ["it is true"])
                        )
                        (call "{1}" ("println" "print") ["it is false"])
                    )
                )
                (call "{0}" ("errorHandlingSrv" "error") [%last_error%])
            )
            "#,
        local_peer_id, local_peer_id_2
    );

    let result = checked_call_vm!(vm, "", script, "", "");

    let mut actual_trace = trace_from_result(&result);
    let expected_executed_call_result = executed_state::request_sent_by(local_peer_id);

    assert_eq!(actual_trace.pop().unwrap(), expected_executed_call_result);
}

// https://github.com/fluencelabs/aquavm/issues/165
#[test]
fn issue_165() {
    let result_setter_peer_id = "result_setter_peer_id";
    let mut result_setter = create_avm(
        set_variable_call_service(serde_json::json!({"success": true})),
        result_setter_peer_id,
    );

    let echo_peer_id = "echo_peer_id";
    let mut echo_peer = create_avm(echo_call_service(), echo_peer_id);

    let script = format!(
        r#"
        (seq
            (call "{0}" ("" "") ["set_result"] result)
            (seq
                (xor
                    (match result.$.success! true
                        (ap 1 $results)
                    )
                    (ap 2 $results)
                )
                (call "{1}" ("callbackSrv" "response") [$results.$.[0]!])
            )
        )
    "#,
        result_setter_peer_id, echo_peer_id
    );

    let setter_result = checked_call_vm!(result_setter, "", &script, "", "");
    let echo_result = checked_call_vm!(echo_peer, "", &script, "", setter_result.data);

    let trace = trace_from_result(&echo_result);
    assert_eq!(trace.last().unwrap(), &executed_state::scalar(json!(1)));
}
