// OpenAI client

use std::time::Duration;

use crate::config::ClientConfig;
use crate::error::{ErrorResponse, OpenAIError};
use crate::resources::audio::Audio;
use crate::resources::chat::Chat;
use crate::resources::embeddings::Embeddings;
use crate::resources::files::Files;
use crate::resources::fine_tuning::FineTuning;
use crate::resources::images::Images;
use crate::resources::models::Models;
use crate::resources::moderations::Moderations;

/// Status codes that trigger a retry.
const RETRYABLE_STATUS_CODES: [u16; 4] = [429, 500, 502, 503];

/// The main OpenAI client.
#[derive(Debug, Clone)]
pub struct OpenAI {
    pub(crate) http: reqwest::Client,
    pub(crate) config: ClientConfig,
}

impl OpenAI {
    /// Create a new client with the given API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self::with_config(ClientConfig::new(api_key))
    }

    /// Create a client from a full config.
    pub fn with_config(config: ClientConfig) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .expect("failed to build HTTP client");
        Self { http, config }
    }

    /// Create a client using the `OPENAI_API_KEY` environment variable.
    pub fn from_env() -> Result<Self, OpenAIError> {
        Ok(Self::with_config(ClientConfig::from_env()?))
    }

    /// Access the Audio resource.
    pub fn audio(&self) -> Audio<'_> {
        Audio::new(self)
    }

    /// Access the Chat resource.
    pub fn chat(&self) -> Chat<'_> {
        Chat::new(self)
    }

    /// Access the Models resource.
    pub fn models(&self) -> Models<'_> {
        Models::new(self)
    }

    /// Access the Fine-tuning resource.
    pub fn fine_tuning(&self) -> FineTuning<'_> {
        FineTuning::new(self)
    }

    /// Access the Files resource.
    pub fn files(&self) -> Files<'_> {
        Files::new(self)
    }

    /// Access the Images resource.
    pub fn images(&self) -> Images<'_> {
        Images::new(self)
    }

    /// Access the Moderations resource.
    pub fn moderations(&self) -> Moderations<'_> {
        Moderations::new(self)
    }

    /// Access the Embeddings resource.
    pub fn embeddings(&self) -> Embeddings<'_> {
        Embeddings::new(self)
    }

    /// Build a request with auth headers.
    pub(crate) fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.config.base_url, path);
        let mut req = self
            .http
            .request(method, &url)
            .bearer_auth(&self.config.api_key);

        if let Some(ref org) = self.config.organization {
            req = req.header("OpenAI-Organization", org);
        }
        if let Some(ref project) = self.config.project {
            req = req.header("OpenAI-Project", project);
        }

        req
    }

    /// Send a GET request and deserialize the response.
    #[allow(dead_code)]
    pub(crate) async fn get<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
    ) -> Result<T, OpenAIError> {
        self.send_with_retry(reqwest::Method::GET, path, None::<&()>)
            .await
    }

    /// Send a POST request with a JSON body and deserialize the response.
    pub(crate) async fn post<B: serde::Serialize, T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, OpenAIError> {
        self.send_with_retry(reqwest::Method::POST, path, Some(body))
            .await
    }

    /// Send a POST request with a multipart form body and deserialize the response.
    pub(crate) async fn post_multipart<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        form: reqwest::multipart::Form,
    ) -> Result<T, OpenAIError> {
        let response = self
            .request(reqwest::Method::POST, path)
            .multipart(form)
            .send()
            .await?;
        Self::handle_response(response).await
    }

    /// Send a GET request and return raw bytes.
    pub(crate) async fn get_raw(&self, path: &str) -> Result<bytes::Bytes, OpenAIError> {
        let response = self.request(reqwest::Method::GET, path).send().await?;

        let status = response.status();
        if status.is_success() {
            Ok(response.bytes().await?)
        } else {
            Err(Self::extract_error(status.as_u16(), response).await)
        }
    }

    /// Send a POST request with JSON body and return raw bytes (for binary responses like audio).
    pub(crate) async fn post_raw<B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<bytes::Bytes, OpenAIError> {
        let response = self
            .request(reqwest::Method::POST, path)
            .json(body)
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            Ok(response.bytes().await?)
        } else {
            Err(Self::extract_error(status.as_u16(), response).await)
        }
    }

    /// Send a DELETE request and deserialize the response.
    #[allow(dead_code)]
    pub(crate) async fn delete<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
    ) -> Result<T, OpenAIError> {
        self.send_with_retry(reqwest::Method::DELETE, path, None::<&()>)
            .await
    }

    /// Send a request with retry logic for transient errors.
    async fn send_with_retry<B: serde::Serialize, T: serde::de::DeserializeOwned>(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<&B>,
    ) -> Result<T, OpenAIError> {
        let max_retries = self.config.max_retries;
        let mut last_error: Option<OpenAIError> = None;

        for attempt in 0..=max_retries {
            let mut req = self.request(method.clone(), path);
            if let Some(b) = body {
                req = req.json(b);
            }

            let response = match req.send().await {
                Ok(resp) => resp,
                Err(e) => {
                    last_error = Some(OpenAIError::RequestError(e));
                    if attempt < max_retries {
                        tokio::time::sleep(Self::backoff_delay(attempt, None)).await;
                        continue;
                    }
                    break;
                }
            };

            let status = response.status().as_u16();

            if !RETRYABLE_STATUS_CODES.contains(&status) || attempt == max_retries {
                return Self::handle_response(response).await;
            }

            // Retryable status — parse Retry-After and sleep
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<f64>().ok());

            last_error = Some(Self::extract_error(status, response).await);
            tokio::time::sleep(Self::backoff_delay(attempt, retry_after)).await;
        }

        Err(last_error.unwrap_or_else(|| {
            OpenAIError::InvalidArgument("retry loop exhausted without error".to_string())
        }))
    }

    /// Calculate backoff delay: max(retry_after, 0.5 * 2^attempt) seconds.
    fn backoff_delay(attempt: u32, retry_after_secs: Option<f64>) -> Duration {
        let exponential = 0.5 * 2.0_f64.powi(attempt as i32);
        let secs = match retry_after_secs {
            Some(ra) => ra.max(exponential),
            None => exponential,
        };
        Duration::from_secs_f64(secs.min(60.0))
    }

    /// Handle API response: check status, parse errors or deserialize body.
    async fn handle_response<T: serde::de::DeserializeOwned>(
        response: reqwest::Response,
    ) -> Result<T, OpenAIError> {
        let status = response.status();
        if status.is_success() {
            let body = response.text().await?;
            let value: T = serde_json::from_str(&body)?;
            Ok(value)
        } else {
            Err(Self::extract_error(status.as_u16(), response).await)
        }
    }

    /// Extract an OpenAIError from a failed response.
    async fn extract_error(status: u16, response: reqwest::Response) -> OpenAIError {
        let body = response.text().await.unwrap_or_default();
        if let Ok(error_resp) = serde_json::from_str::<ErrorResponse>(&body) {
            OpenAIError::ApiError {
                status,
                message: error_resp.error.message,
                type_: error_resp.error.type_,
                code: error_resp.error.code,
            }
        } else {
            OpenAIError::ApiError {
                status,
                message: body,
                type_: None,
                code: None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_client() {
        let client = OpenAI::new("sk-test-key");
        assert_eq!(client.config.api_key, "sk-test-key");
        assert_eq!(client.config.base_url, "https://api.openai.com/v1");
    }

    #[test]
    fn test_with_config() {
        let config = ClientConfig::new("sk-test")
            .base_url("https://custom.api.com")
            .organization("org-123")
            .timeout_secs(30);
        let client = OpenAI::with_config(config);
        assert_eq!(client.config.base_url, "https://custom.api.com");
        assert_eq!(client.config.organization.as_deref(), Some("org-123"));
        assert_eq!(client.config.timeout_secs, 30);
    }

    #[test]
    fn test_backoff_delay() {
        // Attempt 0: 0.5s
        let d = OpenAI::backoff_delay(0, None);
        assert_eq!(d, Duration::from_millis(500));

        // Attempt 1: 1.0s
        let d = OpenAI::backoff_delay(1, None);
        assert_eq!(d, Duration::from_secs(1));

        // Attempt 2: 2.0s
        let d = OpenAI::backoff_delay(2, None);
        assert_eq!(d, Duration::from_secs(2));

        // Retry-After takes precedence when larger
        let d = OpenAI::backoff_delay(0, Some(5.0));
        assert_eq!(d, Duration::from_secs(5));

        // Exponential wins when larger than Retry-After
        let d = OpenAI::backoff_delay(3, Some(0.1));
        assert_eq!(d, Duration::from_secs(4));

        // Capped at 60s
        let d = OpenAI::backoff_delay(10, None);
        assert_eq!(d, Duration::from_secs(60));
    }

    #[tokio::test]
    async fn test_get_success() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/models/gpt-4")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{"id":"gpt-4","object":"model","created":1687882411,"owned_by":"openai"}"#,
            )
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));

        #[derive(serde::Deserialize)]
        struct Model {
            id: String,
            object: String,
        }

        let model: Model = client.get("/models/gpt-4").await.unwrap();
        assert_eq!(model.id, "gpt-4");
        assert_eq!(model.object, "model");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_post_success() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/chat/completions")
            .match_header("authorization", "Bearer sk-test")
            .match_header("content-type", "application/json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":"chatcmpl-123","object":"chat.completion"}"#)
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));

        #[derive(serde::Serialize)]
        struct Req {
            model: String,
        }
        #[derive(serde::Deserialize)]
        struct Resp {
            id: String,
        }

        let resp: Resp = client
            .post(
                "/chat/completions",
                &Req {
                    model: "gpt-4".into(),
                },
            )
            .await
            .unwrap();
        assert_eq!(resp.id, "chatcmpl-123");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_delete_success() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("DELETE", "/models/ft-abc")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":"ft-abc","deleted":true}"#)
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));

        #[derive(serde::Deserialize)]
        struct DeleteResp {
            id: String,
            deleted: bool,
        }

        let resp: DeleteResp = client.delete("/models/ft-abc").await.unwrap();
        assert_eq!(resp.id, "ft-abc");
        assert!(resp.deleted);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_api_error_response() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/models/nonexistent")
            .with_status(404)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{"error":{"message":"The model 'nonexistent' does not exist","type":"invalid_request_error","param":null,"code":"model_not_found"}}"#,
            )
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));

        #[derive(Debug, serde::Deserialize)]
        struct Model {
            _id: String,
        }

        let err = client
            .get::<Model>("/models/nonexistent")
            .await
            .unwrap_err();
        match err {
            OpenAIError::ApiError {
                status,
                message,
                type_,
                code,
            } => {
                assert_eq!(status, 404);
                assert!(message.contains("does not exist"));
                assert_eq!(type_.as_deref(), Some("invalid_request_error"));
                assert_eq!(code.as_deref(), Some("model_not_found"));
            }
            other => panic!("expected ApiError, got: {other:?}"),
        }
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_auth_headers() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/test")
            .match_header("authorization", "Bearer sk-key")
            .match_header("OpenAI-Organization", "org-abc")
            .match_header("OpenAI-Project", "proj-xyz")
            .with_status(200)
            .with_body(r#"{"ok":true}"#)
            .create_async()
            .await;

        let client = OpenAI::with_config(
            ClientConfig::new("sk-key")
                .base_url(server.url())
                .organization("org-abc")
                .project("proj-xyz"),
        );

        #[derive(serde::Deserialize)]
        struct Resp {
            ok: bool,
        }

        let resp: Resp = client.get("/test").await.unwrap();
        assert!(resp.ok);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_retry_on_429_then_success() {
        let mut server = mockito::Server::new_async().await;

        // First request returns 429, second returns 200
        let _mock_429 = server
            .mock("GET", "/test")
            .with_status(429)
            .with_header("retry-after", "0")
            .with_body(r#"{"error":{"message":"Rate limited","type":"rate_limit_error","param":null,"code":null}}"#)
            .create_async()
            .await;

        let mock_200 = server
            .mock("GET", "/test")
            .with_status(200)
            .with_body(r#"{"ok":true}"#)
            .create_async()
            .await;

        let client = OpenAI::with_config(
            ClientConfig::new("sk-test")
                .base_url(server.url())
                .max_retries(2),
        );

        #[derive(serde::Deserialize)]
        struct Resp {
            ok: bool,
        }

        let resp: Resp = client.get("/test").await.unwrap();
        assert!(resp.ok);
        mock_200.assert_async().await;
    }

    #[tokio::test]
    async fn test_retry_exhausted_returns_last_error() {
        let mut server = mockito::Server::new_async().await;

        // All requests return 500
        let _mock = server
            .mock("GET", "/test")
            .with_status(500)
            .with_body(r#"{"error":{"message":"Internal server error","type":"server_error","param":null,"code":null}}"#)
            .expect_at_least(2)
            .create_async()
            .await;

        let client = OpenAI::with_config(
            ClientConfig::new("sk-test")
                .base_url(server.url())
                .max_retries(1),
        );

        #[derive(Debug, serde::Deserialize)]
        struct Resp {
            _ok: bool,
        }

        let err = client.get::<Resp>("/test").await.unwrap_err();
        match err {
            OpenAIError::ApiError { status, .. } => assert_eq!(status, 500),
            other => panic!("expected ApiError, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_no_retry_on_400() {
        let mut server = mockito::Server::new_async().await;

        // 400 should not be retried
        let mock = server
            .mock("GET", "/test")
            .with_status(400)
            .with_body(r#"{"error":{"message":"Bad request","type":"invalid_request_error","param":null,"code":null}}"#)
            .expect(1)
            .create_async()
            .await;

        let client = OpenAI::with_config(
            ClientConfig::new("sk-test")
                .base_url(server.url())
                .max_retries(2),
        );

        #[derive(Debug, serde::Deserialize)]
        struct Resp {
            _ok: bool,
        }

        let err = client.get::<Resp>("/test").await.unwrap_err();
        match err {
            OpenAIError::ApiError { status, .. } => assert_eq!(status, 400),
            other => panic!("expected ApiError, got: {other:?}"),
        }
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_zero_retries_no_retry() {
        let mut server = mockito::Server::new_async().await;

        let mock = server
            .mock("GET", "/test")
            .with_status(429)
            .with_body(r#"{"error":{"message":"Rate limited","type":"rate_limit_error","param":null,"code":null}}"#)
            .expect(1)
            .create_async()
            .await;

        let client = OpenAI::with_config(
            ClientConfig::new("sk-test")
                .base_url(server.url())
                .max_retries(0),
        );

        #[derive(Debug, serde::Deserialize)]
        struct Resp {
            _ok: bool,
        }

        let err = client.get::<Resp>("/test").await.unwrap_err();
        match err {
            OpenAIError::ApiError { status, .. } => assert_eq!(status, 429),
            other => panic!("expected ApiError, got: {other:?}"),
        }
        mock.assert_async().await;
    }
}
