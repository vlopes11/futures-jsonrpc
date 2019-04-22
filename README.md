# futures-jsonrpc

[![Crate](https://img.shields.io/crates/v/futures-jsonrpc.svg)](https://crates.io/crates/futures-jsonrpc)
[![Documentation](https://docs.rs/futures-jsonrpc/badge.svg)](https://docs.rs/futures-jsonrpc)
[![Travis Status](https://travis-ci.org/vlopes11/futures-jsonrpc.svg?branch=master)](https://travis-ci.org/vlopes11/futures-jsonrpc)
[![Join the chat at https://gitter.im/futures-jsonrpc/community](https://badges.gitter.im/futures-jsonrpc/community.svg)](https://gitter.im/futures-jsonrpc/community?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)

## Futures + JSON-RPC

A lightweight remote procedure call protocol. It is designed to be simple! And, with futures, even more flexible!

This crate will associate [Future](futures::future::Future)s with method signatures via [register_method](handler::JrpcHandler::register_method), and parse/handle JSON-RPC messages via [handle_message](handler::JrpcHandler::handle_message).

It is fully compliant with [JSON-RPC 2.0 Specification](https://www.jsonrpc.org/specification).

## Contributing

The crate is not test covered yet! Any PR with a test coverage will be much appreciated :)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
futures_jsonrpc = "0.2"
```

## Minimal example

```rust
use futures_jsonrpc::futures::prelude::*;
use futures_jsonrpc::*;
use serde_json::Number;

// This macro will avoid some boilerplating, leaving only the `Future` implementation to be done
//
// Check for additional information in the detailed explanation below
//
// Also, check `generate_method_with_data_and_future` and `generate_method_with_lifetime_data_and_future`
generate_method!(
    CopyParams,
    impl Future for CopyParams {
        type Item = Option<JrpcResponse>;
        type Error = ErrorVariant;

        fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
            let request = self.get_request()?;
            let params = request.get_params().clone().unwrap_or(JsonValue::Null);

            let message = JrpcResponseParam::generate_result(params)
                .and_then(|result| request.generate_response(result))?;

            Ok(Async::Ready(Some(message)))
        }
    }
);

fn main() {
    // `JrpcHandler` instance is responsible for registering the JSON-RPC methods and receiving the
    // requests.
    //
    // This is full `Arc`/`RwLock` protected. Therefore, it can be freely copied/sent among
    // threads.
    let handler = JrpcHandler::new().unwrap();

    handler
        // `register_method` will tie the method signature to an instance, not a generic. This
        // means we can freely mutate this instance across different signatures.
        .register_method("some/copyParams", CopyParams::new().unwrap())

        .and_then(|h| {
            // `handle_message` will receive a raw implementation of `ToString` and return the
            // associated future. If no future is found, an instance of
            // `Err(ErrorVariant::MethodSignatureNotFound(String))` is returned
            h.handle_message(
                r#"
                {
                    "jsonrpc": "2.0",
                    "method": "some/copyParams",
                    "params": [42, 23],
                    "id": 531
                }"#,
            )
        })

        // Just waiting for the poll of future. Check futures documentation.
        .and_then(|future| future.wait())
        .and_then(|result| {
            // The result is an instance of `JrpcResponse`
            let result = result.unwrap();

            assert_eq!(result.get_jsonrpc(), "2.0");
            assert_eq!(
                result.get_result(),
                &Some(JsonValue::Array(vec![
                    JsonValue::Number(Number::from(42)),
                    JsonValue::Number(Number::from(23)),
                ]))
            );
            assert!(result.get_error().is_none());
            assert_eq!(result.get_id(), &JsonValue::Number(Number::from(531)));
            Ok(())
        })
        .unwrap();
}
```

## Detailed explanation

```rust
use futures_jsonrpc::futures::prelude::*;
use futures_jsonrpc::*;
use std::marker::PhantomData;

// `JrpcHandler` use foreign structures as controllers
// This example will reflect `generate_method_with_lifetime_data_and_future` macro
#[derive(Debug, Clone)]
pub struct CopyParams<'r> {
    request: Option<JrpcRequest>,
    data: (String, i32, PhantomData<&'r ()>),
}

// This implementation is essentially some boilerplate to hold the data that may be used by the
// future poll
impl<'r> CopyParams<'r> {
    // The `new` method will always receive a n-tuple as parameter to store data
    //
    // It is recommended to use atomic types, or `Arc` protected for heavy data. At every request,
    // we `Clone` this struct to send it to the responsible thread
    pub fn new(data: (String, i32, PhantomData<&'r ()>)) -> Result<Self, ErrorVariant> {
        let request = None;
        let some_notification = CopyParams { request, data };
        Ok(some_notification)
    }

    // The `get_data` will support the future poll with additional information that will not be
    // available in the JsonRpc request
    pub fn get_data(&self) -> &(String, i32, PhantomData<&'r ()>) {
        &self.data
    }

    // The `get_request` method will return the JsonRpc request to the future poll
    pub fn get_request(&self) -> Result<JrpcRequest, ErrorVariant> {
        let request = self.request.clone();
        request
            .map(|r| Ok(r.clone()))
            .unwrap_or(Err(ErrorVariant::NoRequestProvided))
    }

    // This method is of internal usage to receive the request from `JrpcHandler`
    pub fn set_request(mut self, request: JrpcRequest) -> Result<Self, ErrorVariant> {
        self.request = Some(request);
        Ok(self)
    }

    // This "fork" will be performed every time a new request is received, allowing async
    // processing
    pub fn clone_with_request(&self, request: JrpcRequest) -> Result<Self, ErrorVariant> {
        self.clone().set_request(request)
    }
}

// `JrpcHandler` will just return a pollable associated future.
//
// The main implementation will go here
//
// Tokio provides very good documentation on futures. Check it: https://tokio.rs/
impl<'r> Future for CopyParams<'r> {
    // Optimally, we want to use JrpcResponse, for it is guaranteed to respect the JSON-RPC
    // specification. But, we can change the response here to something else, if required.
    type Item = Option<JrpcResponse>;
    type Error = ErrorVariant;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        // We fetch the provided request to copy the data
        let request = self.get_request()?;

        // Here we can receive additional that that's not available in the request
        let (_text, _value, _) = self.get_data();

        // Do something with the request
        // In this example, we are copying the parameters
        let params = request.get_params().clone().unwrap_or(JsonValue::Null);

        // `generate_response` will receive an enum `JrpcResponseParam` and reply
        // with either an error or success.
        let message = JrpcResponseParam::generate_result(params)
            .and_then(|result| request.generate_response(result))?;

        // Then, our reply is ready
        Ok(Async::Ready(Some(message)))
    }
}

// The handler will call this trait to spawn a new future and process it when a registered method
// is requested.
impl<'r> JrpcMethodTrait<'r> for CopyParams<'r> {
    // `generate_future` can generate any `Future` that respects the trait signature. This can be a
    // foreign structure, or just a copy of `self`, in case it implements `Future`. This can also
    // be a decision based on the received `JrpcRequest`.
    //
    // Since its not a reference, there are no restrictions.
    fn generate_future(
        &self,
        request: JrpcRequest,
    ) -> Result<Box<'r + Future<Item = Option<JrpcResponse>, Error = ErrorVariant>>, ErrorVariant>
    {
        Ok(Box::new(self.clone_with_request(request)?))
    }
}
```

