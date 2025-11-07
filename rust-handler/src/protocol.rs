use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Request {
    pub id: String,
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct Response {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorObj>,
}

#[derive(Debug, Serialize)]
pub struct ErrorObj {
    pub code: String,
    pub message: String,
}

// Standard error codes
pub const ERR_INVALID_REQUEST: &str = "INVALID_REQUEST";
pub const ERR_METHOD_NOT_FOUND: &str = "METHOD_NOT_FOUND";
pub const ERR_INVALID_PARAMS: &str = "INVALID_PARAMS";
pub const ERR_KERNEL: &str = "KERNEL_ERROR";
pub const ERR_SCRIPT_VERIFY: &str = "SCRIPT_VERIFY_ERROR";
pub const ERR_INTERNAL: &str = "INTERNAL_ERROR";

impl Response {
    pub fn success(id: String, result: serde_json::Value) -> Self {
        Response {
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: String, code: &str, message: String) -> Self {
        Response {
            id,
            result: None,
            error: Some(ErrorObj {
                code: code.to_string(),
                message,
            }),
        }
    }
}
