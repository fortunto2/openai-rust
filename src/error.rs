// OpenAI API error types

use serde::{Deserialize, Serialize};

/// Error response body from the OpenAI API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ApiErrorDetail,
}

/// Detail within an API error response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorDetail {
    pub message: String,
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub param: Option<String>,
    pub code: Option<String>,
}

/// All errors that can occur when using this library.
#[derive(Debug, thiserror::Error)]
pub enum OpenAIError {
    /// The API returned an error response.
    #[error("API error (status {status}): {message}")]
    ApiError {
        status: u16,
        message: String,
        type_: Option<String>,
        code: Option<String>,
    },

    /// HTTP request failed.
    #[error("request error: {0}")]
    RequestError(#[from] reqwest::Error),

    /// JSON (de)serialization failed.
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// SSE stream error.
    #[error("stream error: {0}")]
    StreamError(String),

    /// Invalid argument passed by the caller.
    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    /// WebSocket error.
    #[cfg(feature = "websocket")]
    #[error("websocket error: {0}")]
    WebSocketError(String),
}
