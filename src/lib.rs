//! Futures + JSON-RPC
//!
//! A lightweight remote procedure call protocol. It is designed to be simple! And, with futures, even more flexible!
//!
//! This crate will associate [Future](futures::future::Future)s with method signatures via [register_method](handler::JrpcHandler::register_method), and parse/handle JSON-RPC messages via [handle_message](handler::JrpcHandler::handle_message).
//!
//! It is fully compliant with [JSON-RPC 2.0 Specification](https://www.jsonrpc.org/specification).
//!
//! ## Installation
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! futures_jsonrpc = "0.1"
//! ```
//!
//! ## Examples
//!
//! ```
//! use futures_jsonrpc::futures::prelude::*;
//! use futures_jsonrpc::*;
//! use serde_json::Number;
//!
//! // `JrpcHandler` use foreign structures as controllers
//! #[derive(Debug, Clone)]
//! struct SomeNotification {
//!     request: Option<JrpcRequest>,
//! }
//!
//! // Here is some boilerplate to instantiate a new Notification, and handle the received request
//! impl SomeNotification {
//!     pub fn new() -> Result<Self, ErrorVariant> {
//!         let request = None;
//!         let some_notification = SomeNotification { request };
//!         Ok(some_notification)
//!     }
//!
//!     pub fn get_request(&self) -> Result<JrpcRequest, ErrorVariant> {
//!         let request = self.request.clone();
//!         request
//!             .map(|r| Ok(r.clone()))
//!             .unwrap_or(Err(ErrorVariant::NoRequestProvided))
//!     }
//!
//!     pub fn set_request(mut self, request: JrpcRequest) -> Result<Self, ErrorVariant> {
//!         self.request = Some(request);
//!         Ok(self)
//!     }
//!
//!     pub fn clone_with_request(&self, request: JrpcRequest) -> Result<Self, ErrorVariant> {
//!         self.clone().set_request(request)
//!     }
//! }
//!
//! // `JrpcHandler` will just return a pollable associated future.
//! //
//! // The main implementation will go here
//! //
//! // Tokio provides very good documentation on futures. Check it: https://tokio.rs/
//! impl Future for SomeNotification {
//!     // Optimally, we want to use JrpcResponse, for it is guaranteed to respect the JSON-RPC
//!     // specification. But, we can change the response here to something else, if required.
//!     type Item = Option<JrpcResponse>;
//!     type Error = ErrorVariant;
//!
//!     fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
//!         // We fetch the provided request to copy the data
//!         let request = self.get_request()?;
//!
//!         // The params, in this case, will be only a reflection on what was sent
//!         let params = request.get_params().clone().unwrap_or(JsonValue::Null);
//!
//!         // `generate_response` will receive an enum `JrpcResponseParam` and reply
//!         // with either an error or success.
//!         let message = JrpcResponseParam::generate_result(params)
//!             .and_then(|result| request.generate_response(result))?;
//!
//!         // Then, our reply is ready
//!         Ok(Async::Ready(Some(message)))
//!     }
//! }
//!
//! // The handler will call this trait to spawn a new future and process it when a registered method
//! // is requested.
//! impl JrpcMethodTrait for SomeNotification {
//!     // `generate_future` can generate any `Future` that respects the trait signature. This can be a
//!     // foreign structure, or just a copy of `self`, in case it implements `Future`. This can also
//!     // be a decision based on the received `JrpcRequest`.
//!     //
//!     // Since its not a reference, there are no restrictions.
//!     fn generate_future<'a>(
//!         &self,
//!         request: JrpcRequest,
//!     ) -> Result<Box<'a + Future<Item = Option<JrpcResponse>, Error = ErrorVariant>>, ErrorVariant>
//!     {
//!         Ok(Box::new(self.clone_with_request(request)?))
//!     }
//! }
//!
//! // Or, alternitavely, we can use the `generate_method_with_future` macro to only implement the
//! // `Future`
//! generate_method_with_future!(InitializeRequest, impl Future for InitializeRequest {
//!     type Item = Option<JrpcResponse>;
//!     type Error = ErrorVariant;
//!
//!     fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
//!         let request = self.get_request()?;
//!
//!         let params = request.get_params().clone().unwrap_or(JsonValue::Null);
//!
//!         let message = JrpcResponseParam::generate_result(params)
//!             .and_then(|result| request.generate_response(result))?;
//!
//!         Ok(Async::Ready(Some(message)))
//!     }
//! });
//!
//! fn main() {
//!     // `JrpcHanlder` instance is responsible for registering the JSON-RPC methods and receiving the
//!     // requests.
//!     //
//!     // This is full `Arc`/`RwLock` protected. Therefore, it can be freely copied/sent among
//!     // threads.
//!     let handler = JrpcHandler::new().unwrap();
//!
//!     handler
//!         // `register_method` will tie the method signature to an instance, not a generic. This
//!         // means we can freely mutate this instance across different signatures.
//!         .register_method("some/copyParams", SomeNotification::new().unwrap())
//!         .and_then(|h| h.register_method("initialize", InitializeRequest::new().unwrap()))
//!         .and_then(|h| {
//!             // `handle_message` will receive a raw implementation of `ToString` and return the
//!             // associated future. If no future is found, an instance of
//!             // `Err(ErrorVariant::MethodSignatureNotFound)` is returned
//!             h.handle_message(
//!                 r#"
//!                 {
//!                     "jsonrpc": "2.0",
//!                     "method": "some/copyParams",
//!                     "params": [42, 23],
//!                     "id": 531
//!                 }"#,
//!             )
//!         })
//!         // Just waiting for the poll of future. Check futures documentation.
//!         .and_then(|future| future.wait())
//!         .and_then(|result| {
//!             // The result is an instance of `JrpcResponse`
//!             let result = result.unwrap();
//!             assert_eq!(result.get_jsonrpc(), "2.0");
//!             assert_eq!(
//!                 result.get_result(),
//!                 &Some(JsonValue::Array(vec![
//!                     JsonValue::Number(Number::from(42)),
//!                     JsonValue::Number(Number::from(23)),
//!                 ]))
//!             );
//!             assert!(result.get_error().is_none());
//!             assert_eq!(result.get_id(), &JsonValue::Number(Number::from(531)));
//!             Ok(())
//!         })
//!         .unwrap();
//! }
//! ```

#[macro_use]
extern crate log;

pub use crate::handler::JrpcHandler;
pub use crate::method::JrpcMethodTrait;
pub use crate::parser::{JrpcError, JrpcErrorEnum, JrpcRequest, JrpcResponse, JrpcResponseParam};
pub use futures;
pub use serde_json::error::Error as JsonError;
pub use serde_json::Value as JsonValue;
use std::fmt;
use std::io::Error as IoError;

pub mod handler;
pub mod method;
pub mod parser;

#[derive(Debug)]
pub enum ErrorVariant {
    RwLockPoisoned,
    MethodSignatureNotFound,
    JsonParseError(JsonError),
    InvalidJsonRpcVersion,
    InvalidJsonRpcId,
    ResponseCannotContainResultAndError,
    ResponseMustContainResultOrError,
    NoRequestProvided,
    IoError(IoError),
}

impl fmt::Display for ErrorVariant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[macro_export]
macro_rules! generate_method_with_future {
    ($struct_identifier:ident, $future:item) => {
        #[derive(Debug, Clone)]
        pub struct $struct_identifier {
            request: Option<JrpcRequest>,
        }

        impl $struct_identifier {
            pub fn new() -> Result<Self, ErrorVariant> {
                let request = None;
                let some_notification = $struct_identifier { request };
                Ok(some_notification)
            }

            pub fn get_request(&self) -> Result<JrpcRequest, ErrorVariant> {
                let request = self.request.clone();
                request
                    .map(|r| Ok(r.clone()))
                    .unwrap_or(Err(ErrorVariant::NoRequestProvided))
            }

            pub fn set_request(mut self, request: JrpcRequest) -> Result<Self, ErrorVariant> {
                self.request = Some(request);
                Ok(self)
            }

            pub fn clone_with_request(&self, request: JrpcRequest) -> Result<Self, ErrorVariant> {
                self.clone().set_request(request)
            }
        }

        $future

        impl JrpcMethodTrait for $struct_identifier {
            fn generate_future<'a>(
                &self,
                request: JrpcRequest,
            ) -> Result<
                Box<'a + Future<Item = Option<JrpcResponse>, Error = ErrorVariant>>,
                ErrorVariant,
            > {
                Ok(Box::new(self.clone_with_request(request)?))
            }
        }
    };
}
