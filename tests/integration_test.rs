use futures_jsonrpc::*;
use futures_jsonrpc::futures::prelude::*;
use serde_json::json;

generate_method!(
    Subtract,

    impl Future for Subtract {
        type Item = Option<JrpcResponse>;
        type Error = ErrorVariant;

        fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
            let request = self.get_request()?;
            let params = request.get_params().clone();

            let value = match params {
                Some(serde_json::Value::Array(ref subs)) if subs.len() == 2 => {
                    let res = match (&subs[0].as_i64(), &subs[1].as_i64()) {
                        (Some(subs0), Some(subs1)) => Some(subs0 - subs1),
                        _ => None,
                    };
                    res
                },
                _ => None,
            };

            let msg = match value {
                Some(v) => JrpcResponseParam::generate_result(json!({"result": v})),
                None => JrpcResponseParam::generate_error(JrpcError::from(JrpcErrorEnum::InvalidParams)),
            };

            let ret = match msg {
                Ok(res) => request.generate_response(res),
                _ => return Err(ErrorVariant::InternalError),
            };

            Ok(Async::Ready(ret.ok()))
        }
    }
);

#[test]
fn subtraction_test() {
    let tests = vec![
        (
            r#"
            {
                "jsonrpc": "2.0",
                "method": "math/subtract",
                "params": [42, 23],
                "id": 1
            }"#,
            serde_json::json!({
                "jsonrpc": "2.0", "result": 19, "id": 1
            })
        ),
        (
            r#"
            {
                "jsonrpc": "2.0",
                "method": "math/subtract",
                "params": [23, 42],
                "id": 2
            }"#,
            serde_json::json!({
                "jsonrpc": "2.0", "result": -19, "id": 2
            })
        )
    ];

    for test in tests {
        let handler = JrpcHandler::new().unwrap();

        handler
            .register_method("math/subtract", Subtract::new().unwrap())
            .and_then(|h| {h.handle_message(test.0)})
            .and_then(|f| f.wait())
            .and_then(|res| {
                let res = res.unwrap();

                assert_eq!(res.get_id().as_i64(), test.1["id"].as_i64());
                match res.get_result() {
                    Some(ref r) => {
                        assert_eq!(r["result"].as_i64(), test.1["result"].as_i64());
                    }
                    _ => assert!(false),

                }

                Ok(())
            })
            .ok();
    }
}

generate_method!(
    SubtractWithNamedParameter,

    impl Future for SubtractWithNamedParameter {
        type Item = Option<JrpcResponse>;
        type Error = ErrorVariant;

        fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
            let request = self.get_request()?;
            let params = request.get_params().clone();

            let value = match params {
                Some(ref subs) => {
                    let res = match (subs["subtrahend"].as_i64(), subs["minuend"].as_i64()) {
                        (Some(subtrahend), Some(minuend)) => Some(minuend - subtrahend),
                        _ => None,
                    };
                    res
                },
                _ => None,
            };

            let msg = match value {
                Some(v) => JrpcResponseParam::generate_result(json!({"result": v})),
                None => JrpcResponseParam::generate_error(JrpcError::from(JrpcErrorEnum::InvalidParams)),
            };

            let ret = match msg {
                Ok(res) => request.generate_response(res),
                _ => return Err(ErrorVariant::InternalError),
            };

            Ok(Async::Ready(ret.ok()))
        }
    }
);

#[test]
fn subtraction_with_named_parameter_test() {
    let tests = vec![
        (
            r#"
            {
                "jsonrpc": "2.0",
                "method": "math/subtract",
                "params": {"subtrahend": 23, "minuend": 42},
                "id": 3
            }"#,
            serde_json::json!({
                "jsonrpc": "2.0", "result": 19, "id": 3
            })
        ),
        (
            r#"
            {
                "jsonrpc": "2.0",
                "method": "math/subtract",
                "params": {"minuend": 23, "subtrahend": 42},
                "id": 4
            }"#,
            serde_json::json!({
                "jsonrpc": "2.0", "result": -19, "id": 4
            })
        )
    ];

    for test in tests {
        let handler = JrpcHandler::new().unwrap();

        handler
            .register_method("math/subtract", SubtractWithNamedParameter::new().unwrap())
            .and_then(|h| {h.handle_message(test.0)})
            .and_then(|f| f.wait())
            .and_then(|res| {
                let res = res.unwrap();

                assert_eq!(res.get_id().as_i64(), test.1["id"].as_i64());
                match res.get_result() {
                    Some(ref r) => {
                        assert_eq!(r["result"].as_i64(), test.1["result"].as_i64());
                    }
                    _ => assert!(false),

                }

                Ok(())
            })
            .map_err(|err| {
                dbg!(err);
                assert!(false);
            })
            .ok();
    }
}

generate_method!(
    Update,

    impl Future for Update {
        type Item = Option<JrpcResponse>;
        type Error = ErrorVariant;

        fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
            let request = self.get_request()?;
            let message = JrpcResponseParam::generate_result(JsonValue::Null)
                .and_then(|result| request.generate_response(result))?;

            Ok(Async::Ready(Some(message)))
        }
    }
);

#[test]
fn notification_test() {
    let tests = vec![
        r#"
        {
            "jsonrpc": "2.0",
            "method": "update",
            "params": [1,2,3,4,5]
        }"#
    ];

    for test in tests {
        let handler = JrpcHandler::new().unwrap();

        handler
            .register_method("update", Update::new().unwrap())
            .and_then(|h| {h.handle_message(test)})
            .and_then(|f| f.wait())
            .and_then(|res| {
                let res = res.unwrap();

                match res.get_result() {
                    Some(_) => (),
                    _ => assert!(false),

                }

                Ok(())
            })
            .map_err(|_| { assert!(false); })
            .ok();
    }
}

#[test]
fn error_test() {
    let tests = vec![
        // no-existent method
        (
            r#"
            {
                "jsonrpc": "2.0",
                "method": "foobar",
                "id": "1"
            }"#,
            serde_json::json!({
                "jsonrpc": "2.0",
                "error": {"code": -32601, "message": "Method not found"},
                "id": "1"
            })
        ),
        // invalid json
        (
            r#"
            {
                "jsonrpc": "2.0",
                "method": "foobar,
                "params": "bar", "baz
            ]"#,
            serde_json::json!({
                "jsonrpc": "2.0",
                "error": {"code": -32700, "message": "Parse error"},
                "id": null
            })
        ),
    ];

    for test in tests {
        let handler = JrpcHandler::new().unwrap();

        handler
            .register_method("update", Update::new().unwrap())
            .and_then(|h| {h.handle_message(test.0)})
            .and_then(|f| f.wait())
            .and_then(|_| {
                assert!(false);
                Ok(())
            })
            .map_err(|err| {
                let ret = *JrpcError::from(err).get_code() as i64;
                let exp = test.1["error"]["code"].as_i64().unwrap();
                assert_eq!(ret, exp);
            })
            .ok();
    }
}
