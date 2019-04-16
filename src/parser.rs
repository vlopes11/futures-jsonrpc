use crate::{ErrorVariant, JsonValue};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JrpcRequest {
    jsonrpc: String,
    method: String,
    params: Option<JsonValue>,
    id: Option<JsonValue>,
}

impl JrpcRequest {
    pub fn new(
        method: String,
        params: Option<JsonValue>,
        id: Option<JsonValue>,
    ) -> Result<JrpcRequest, ErrorVariant> {
        let jrpc_request = JrpcRequest {
            jsonrpc: "2.0".to_string(),
            method,
            params,
            id,
        };

        jrpc_request.validate()
    }

    pub fn parse<F: ToString>(message: F) -> Result<Self, ErrorVariant> {
        let message = message.to_string();
        serde_json::from_str::<Self>(message.as_str())
            .map_err(|e| ErrorVariant::JsonParseError(e))
            .and_then(|parsed| parsed.validate())
    }

    pub fn generate_response(
        &self,
        response: JrpcResponseParam,
    ) -> Result<JrpcResponse, ErrorVariant> {
        JrpcResponse::from_jrpc_request(self, response)
    }

    fn validate(self) -> Result<Self, ErrorVariant> {
        if self.get_jsonrpc() != "2.0" {
            return Err(ErrorVariant::InvalidJsonRpcVersion);
        }

        // https://www.jsonrpc.org/specification#id1
        match self.get_id() {
            Some(JsonValue::String(_)) => (),
            Some(JsonValue::Number(_)) => (),
            Some(JsonValue::Null) => (),
            None => (),
            _ => return Err(ErrorVariant::InvalidJsonRpcId),
        }

        Ok(self)
    }

    pub fn is_notification(&self) -> bool {
        self.id.is_none()
    }

    pub fn get_jsonrpc(&self) -> &String {
        &self.jsonrpc
    }

    pub fn get_method(&self) -> &String {
        &self.method
    }

    pub fn get_params(&self) -> &Option<JsonValue> {
        &self.params
    }

    pub fn get_id(&self) -> &Option<JsonValue> {
        &self.id
    }
}

#[derive(Debug, Clone)]
pub enum JrpcResponseParam {
    JrpcResult(JsonValue),
    JrpcError(JrpcError),
}

impl JrpcResponseParam {
    pub fn generate_result(response: JsonValue) -> Result<Self, ErrorVariant> {
        Ok(JrpcResponseParam::JrpcResult(response))
    }

    pub fn generate_error(response: JrpcError) -> Result<Self, ErrorVariant> {
        Ok(JrpcResponseParam::JrpcError(response))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JrpcResponse {
    jsonrpc: String,
    result: Option<JsonValue>,
    error: Option<JrpcError>,
    id: JsonValue,
}

impl JrpcResponse {
    pub fn new(
        result: Option<JsonValue>,
        error: Option<JrpcError>,
        id: JsonValue,
    ) -> Result<Self, ErrorVariant> {
        let jrpc_response = JrpcResponse {
            jsonrpc: "2.0".to_string(),
            result,
            error,
            id,
        };

        jrpc_response.validate()
    }

    pub fn from_jrpc_request(
        request: &JrpcRequest,
        response: JrpcResponseParam,
    ) -> Result<Self, ErrorVariant> {
        let mut result = None;
        let mut error = None;
        let id = match request.get_id() {
            Some(i) => i.clone(),
            None => JsonValue::Null,
        };

        match response {
            JrpcResponseParam::JrpcResult(r) => result = Some(r),
            JrpcResponseParam::JrpcError(e) => error = Some(e),
        }

        JrpcResponse::new(result, error, id)
    }

    pub fn parse<F: ToString>(message: F) -> Result<Self, ErrorVariant> {
        let message = message.to_string();
        serde_json::from_str::<Self>(message.as_str())
            .map_err(|e| ErrorVariant::JsonParseError(e))
            .and_then(|parsed| parsed.validate())
    }

    pub fn validate(self) -> Result<Self, ErrorVariant> {
        if self.get_jsonrpc() != "2.0" {
            return Err(ErrorVariant::InvalidJsonRpcVersion);
        }

        if self.get_result().is_some() && self.get_error().is_some() {
            return Err(ErrorVariant::ResponseCannotContainResultAndError);
        }

        if self.get_result().is_none() && self.get_error().is_none() {
            return Err(ErrorVariant::ResponseCannotContainResultAndError);
        }

        // https://www.jsonrpc.org/specification#id1
        match self.get_id() {
            JsonValue::String(_) => (),
            JsonValue::Number(_) => (),
            JsonValue::Null => (),
            _ => return Err(ErrorVariant::InvalidJsonRpcId),
        }

        Ok(self)
    }

    pub fn get_jsonrpc(&self) -> &String {
        &self.jsonrpc
    }

    pub fn get_result(&self) -> &Option<JsonValue> {
        &self.result
    }

    pub fn get_error(&self) -> &Option<JrpcError> {
        &self.error
    }

    pub fn get_id(&self) -> &JsonValue {
        &self.id
    }
}

#[derive(Debug, Clone)]
pub enum JrpcErrorEnum {
    ParseError = -32700,
    InvalidRequest = -32600,
    MethodNotFound = -32601,
    InvalidParams = -32602,
    InternalError = -32603,
    ServerError = -32000,
    Other = 0,
}

impl From<i32> for JrpcErrorEnum {
    fn from(code: i32) -> Self {
        if code == -32700 {
            JrpcErrorEnum::ParseError
        } else if code == -32600 {
            JrpcErrorEnum::InvalidRequest
        } else if code == -32601 {
            JrpcErrorEnum::MethodNotFound
        } else if code == -32602 {
            JrpcErrorEnum::InvalidParams
        } else if code == -32603 {
            JrpcErrorEnum::InternalError
        } else if code >= -32099 && code <= -32000 {
            JrpcErrorEnum::ServerError
        } else {
            JrpcErrorEnum::Other
        }
    }
}

impl From<JrpcErrorEnum> for i32 {
    fn from(jrpc_error: JrpcErrorEnum) -> i32 {
        match jrpc_error {
            JrpcErrorEnum::ParseError => -32700,
            JrpcErrorEnum::InvalidRequest => -32600,
            JrpcErrorEnum::MethodNotFound => -32601,
            JrpcErrorEnum::InvalidParams => -32602,
            JrpcErrorEnum::InternalError => -32603,
            JrpcErrorEnum::ServerError => -32000,
            JrpcErrorEnum::Other => 0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JrpcError {
    code: i32,
    message: String,
    data: Option<JsonValue>,
}

impl JrpcError {
    pub fn new(code: i32, message: String, data: Option<JsonValue>) -> Self {
        JrpcError {
            code,
            message,
            data,
        }
    }

    pub fn parse<F: ToString>(message: F) -> Result<Self, ErrorVariant> {
        let message = message.to_string();
        let parsed: JrpcError =
            serde_json::from_str(message.as_str()).map_err(|e| ErrorVariant::JsonParseError(e))?;

        Ok(parsed)
    }

    pub fn get_code(&self) -> &i32 {
        &self.code
    }

    pub fn get_message(&self) -> &String {
        &self.message
    }

    pub fn get_data(&self) -> &Option<JsonValue> {
        &self.data
    }
}

impl From<JrpcErrorEnum> for JrpcError {
    fn from(error_enum: JrpcErrorEnum) -> Self {
        let message = match error_enum {
            JrpcErrorEnum::ParseError => "Invalid JSON was received by the server. An error occurred on the server while parsing the JSON text.",
            JrpcErrorEnum::InvalidRequest => "The JSON sent is not a valid Request object.",
            JrpcErrorEnum::MethodNotFound => "The method does not exist / is not available.",
            JrpcErrorEnum::InvalidParams => "Invalid method parameter(s).",
            JrpcErrorEnum::InternalError => "Internal JSON-RPC error.",
            JrpcErrorEnum::ServerError => "Reserved for implementation-defined server-errors.",
            JrpcErrorEnum::Other => "JsonRpc Error",
        };
        let code = i32::from(error_enum);
        let message = message.to_string();
        let data = None;
        JrpcError::new(code, message, data)
    }
}

impl From<i32> for JrpcError {
    fn from(error_code: i32) -> Self {
        let jrpc_error_enum = JrpcErrorEnum::from(error_code);
        JrpcError::from(jrpc_error_enum)
    }
}

impl From<ErrorVariant> for JrpcError {
    fn from(error_variant: ErrorVariant) -> Self {
        match error_variant {
            ErrorVariant::MethodSignatureNotFound => JrpcError::from(-32601),
            ErrorVariant::JsonParseError(_) => JrpcError::from(-32700),
            ErrorVariant::InvalidJsonRpcVersion => JrpcError::from(-32600),
            ErrorVariant::InvalidJsonRpcId => JrpcError::from(-32600),
            ErrorVariant::ResponseCannotContainResultAndError => JrpcError::from(-32600),
            ErrorVariant::ResponseMustContainResultOrError => JrpcError::from(-32600),
            _ => JrpcError::from(-32603),
        }
    }
}
