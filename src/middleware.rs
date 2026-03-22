//! Middleware and interceptor system.
//!
//! Provides a hook mechanism to intercept outgoing requests and incoming responses.
//! This allows implementing logging, metrics, retry telemetry, and rate limit tracking.

use reqwest::{Request, Response};
use std::fmt::Debug;

/// Middleware trait for intercepting requests and responses.
#[async_trait::async_trait]
pub trait Middleware: Send + Sync + Debug {
    /// Called before the request is dispatched.
    async fn on_request(&self, _req: &mut Request) -> Result<(), crate::error::OpenAIError> {
        Ok(())
    }

    /// Called after the response is received.
    async fn on_response(&self, _res: &Response) -> Result<(), crate::error::OpenAIError> {
        Ok(())
    }
}
