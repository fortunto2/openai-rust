// OpenAI client

use std::time::Duration;

use crate::azure::AzureConfig;
use crate::config::ClientConfig;
use crate::error::{ErrorResponse, OpenAIError};
use crate::request_options::RequestOptions;
#[cfg(feature = "audio")]
use crate::resources::audio::Audio;
#[cfg(feature = "batches")]
use crate::resources::batches::Batches;
#[cfg(feature = "beta")]
use crate::resources::beta::assistants::Assistants;
#[cfg(feature = "beta")]
use crate::resources::beta::realtime::Realtime;
#[cfg(feature = "beta")]
use crate::resources::beta::runs::Runs;
#[cfg(feature = "beta")]
use crate::resources::beta::threads::Threads;
#[cfg(feature = "beta")]
use crate::resources::beta::vector_stores::VectorStores;
#[cfg(feature = "chat")]
use crate::resources::chat::Chat;
#[cfg(feature = "embeddings")]
use crate::resources::embeddings::Embeddings;
#[cfg(feature = "files")]
use crate::resources::files::Files;
#[cfg(feature = "fine-tuning")]
use crate::resources::fine_tuning::FineTuning;
#[cfg(feature = "images")]
use crate::resources::images::Images;
#[cfg(feature = "models")]
use crate::resources::models::Models;
#[cfg(feature = "moderations")]
use crate::resources::moderations::Moderations;
#[cfg(feature = "responses")]
use crate::resources::responses::Responses;
#[cfg(feature = "uploads")]
use crate::resources::uploads::Uploads;

/// Status codes that trigger a retry.
const RETRYABLE_STATUS_CODES: [u16; 4] = [429, 500, 502, 503];

/// The main OpenAI client.
///
/// See [OpenAI API docs](https://platform.openai.com/docs/api-reference) for the full API reference.
///
/// Use [`with_options()`](Self::with_options) to create a cheap clone with
/// per-request customization (extra headers, query params, timeout):
///
/// ```ignore
/// use openai_oxide::RequestOptions;
/// use std::time::Duration;
///
/// let custom = client.with_options(
///     RequestOptions::new()
///         .header("X-Custom", "value")
///         .timeout(Duration::from_secs(30))
/// );
/// ```
#[derive(Debug, Clone)]
pub struct OpenAI {
    pub(crate) http: reqwest::Client,
    pub(crate) config: std::sync::Arc<dyn crate::config::Config>,
    pub(crate) options: RequestOptions,
}

impl OpenAI {
    /// Create a new client with the given API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self::with_config(ClientConfig::new(api_key))
    }

    /// Create a client from a full config.
    pub fn with_config<C: crate::config::Config + 'static>(config: C) -> Self {
        let options = config.initial_options();

        #[cfg(not(target_arch = "wasm32"))]
        let http = {
            crate::ensure_tls_provider();

            reqwest::Client::builder()
                .timeout(Duration::from_secs(config.timeout_secs()))
                .tcp_nodelay(true)
                .tcp_keepalive(Some(Duration::from_secs(30)))
                .pool_idle_timeout(Some(Duration::from_secs(300)))
                .pool_max_idle_per_host(4)
                .http2_keep_alive_interval(Some(Duration::from_secs(20)))
                .http2_keep_alive_timeout(Duration::from_secs(10))
                .http2_keep_alive_while_idle(true)
                .http2_adaptive_window(true)
                .gzip(true)
                .build()
                .expect("failed to build HTTP client")
        };

        #[cfg(target_arch = "wasm32")]
        let http = reqwest::Client::new();
        Self {
            http,
            config: std::sync::Arc::new(config),
            options,
        }
    }

    /// Create a cheap clone of this client with additional request options.
    ///
    /// The returned client shares the same HTTP connection pool (`reqwest::Client`
    /// uses `Arc` internally) but applies the merged options to every request.
    ///
    /// ```ignore
    /// use openai_oxide::RequestOptions;
    ///
    /// let custom = client.with_options(
    ///     RequestOptions::new().header("X-Custom", "value")
    /// );
    /// // All requests through `custom` will include the X-Custom header.
    /// let resp = custom.chat().completions().create(req).await?;
    /// ```
    #[must_use]
    pub fn with_options(&self, options: RequestOptions) -> Self {
        Self {
            http: self.http.clone(),
            config: self.config.clone(),
            options: self.options.merge(&options),
        }
    }

    /// Create a client using the `OPENAI_API_KEY` environment variable.
    pub fn from_env() -> Result<Self, OpenAIError> {
        Ok(Self::with_config(ClientConfig::from_env()?))
    }

    /// Create a client configured for Azure OpenAI.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use openai_oxide::{OpenAI, AzureConfig};
    ///
    /// let client = OpenAI::azure(
    ///     AzureConfig::new()
    ///         .azure_endpoint("https://my-resource.openai.azure.com")
    ///         .azure_deployment("gpt-4")
    ///         .api_key("my-azure-key")
    /// )?;
    /// ```
    pub fn azure(config: AzureConfig) -> Result<Self, OpenAIError> {
        config.build()
    }

    /// Access the Batches resource.
    #[cfg(feature = "batches")]
    pub fn batches(&self) -> Batches<'_> {
        Batches::new(self)
    }

    /// Access the Uploads resource.
    #[cfg(feature = "uploads")]
    pub fn uploads(&self) -> Uploads<'_> {
        Uploads::new(self)
    }

    /// Access the Beta resources (Assistants, Threads, Runs, Vector Stores).
    #[cfg(feature = "beta")]
    pub fn beta(&self) -> Beta<'_> {
        Beta { client: self }
    }

    /// Access the Audio resource.
    #[cfg(feature = "audio")]
    pub fn audio(&self) -> Audio<'_> {
        Audio::new(self)
    }

    /// Access the Chat resource.
    #[cfg(feature = "chat")]
    pub fn chat(&self) -> Chat<'_> {
        Chat::new(self)
    }

    /// Access the Models resource.
    #[cfg(feature = "models")]
    pub fn models(&self) -> Models<'_> {
        Models::new(self)
    }

    /// Access the Fine-tuning resource.
    #[cfg(feature = "fine-tuning")]
    pub fn fine_tuning(&self) -> FineTuning<'_> {
        FineTuning::new(self)
    }

    /// Access the Files resource.
    #[cfg(feature = "files")]
    pub fn files(&self) -> Files<'_> {
        Files::new(self)
    }

    /// Access the Images resource.
    #[cfg(feature = "images")]
    pub fn images(&self) -> Images<'_> {
        Images::new(self)
    }

    /// Access the Moderations resource.
    #[cfg(feature = "moderations")]
    pub fn moderations(&self) -> Moderations<'_> {
        Moderations::new(self)
    }

    /// Access the Responses resource.
    #[cfg(feature = "responses")]
    pub fn responses(&self) -> Responses<'_> {
        Responses::new(self)
    }

    /// Access the Embeddings resource.
    #[cfg(feature = "embeddings")]
    pub fn embeddings(&self) -> Embeddings<'_> {
        Embeddings::new(self)
    }

    /// Create a persistent WebSocket session to the Responses API.
    ///
    /// Opens a connection to `wss://api.openai.com/v1/responses` and returns
    /// a [`WsSession`](crate::websocket::WsSession) for low-latency,
    /// multi-turn interactions.
    ///
    /// Requires the `websocket` feature.
    ///
    /// ```ignore
    /// let mut session = client.ws_session().await?;
    /// let response = session.send(request).await?;
    /// session.close().await?;
    /// ```
    #[cfg(feature = "websocket")]
    pub async fn ws_session(&self) -> Result<crate::websocket::WsSession, OpenAIError> {
        crate::websocket::WsSession::connect(self.config.as_ref()).await
    }

    /// Build a request with auth headers and client-level options applied.
    pub(crate) fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.config.base_url(), path);
        let req = self.http.request(method, &url);
        let mut req = self.config.build_request(req);

        // Apply client-level options
        if let Some(ref headers) = self.options.headers {
            for (key, value) in headers.iter() {
                req = req.header(key.clone(), value.clone());
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(ref query) = self.options.query {
            req = req.query(query);
        }
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(timeout) = self.options.timeout {
            req = req.timeout(timeout);
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

    /// Send a GET request with query parameters and deserialize the response.
    #[allow(dead_code)]
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) async fn get_with_query<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        query: &[(String, String)],
    ) -> Result<T, OpenAIError> {
        let mut req = self.request(reqwest::Method::GET, path);
        if !query.is_empty() {
            req = req.query(query);
        }
        let response = req.send().await?;
        Self::handle_response(response).await
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

    /// Send a POST request with a JSON body and return the raw JSON value.
    ///
    /// This is the backbone for BYOT (bring your own types) `create_raw()` methods:
    /// accepts any `Serialize` request and returns `serde_json::Value` instead of a
    /// typed response, letting advanced users work with custom or untyped payloads.
    pub(crate) async fn post_json<B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<serde_json::Value, OpenAIError> {
        self.post(path, body).await
    }

    /// Send a POST request with a multipart form body and deserialize the response.
    #[cfg(not(target_arch = "wasm32"))]
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
        let mut req = self.request(reqwest::Method::POST, path);
        if self.options.extra_body.is_some() {
            req = req.json(&self.merge_body_json(body)?);
        } else {
            req = req.json(body);
        }
        let response = req.send().await?;

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

    /// Serialize body to JSON and merge extra_body fields if set.
    fn merge_body_json<B: serde::Serialize>(
        &self,
        body: &B,
    ) -> Result<serde_json::Value, OpenAIError> {
        let mut value = serde_json::to_value(body)?;
        if let Some(ref extra) = self.options.extra_body
            && let serde_json::Value::Object(map) = &mut value
            && let serde_json::Value::Object(extra_map) = extra.clone()
        {
            for (k, v) in extra_map {
                map.insert(k, v);
            }
        }
        Ok(value)
    }

    /// Pre-serialize request body, merging extra_body if set.
    fn prepare_body<B: serde::Serialize>(
        &self,
        body: Option<&B>,
    ) -> Result<Option<serde_json::Value>, OpenAIError> {
        match body {
            Some(b) if self.options.extra_body.is_some() => Ok(Some(self.merge_body_json(b)?)),
            Some(b) => Ok(Some(serde_json::to_value(b)?)),
            None => Ok(None),
        }
    }

    /// WASM: retry with cross-platform sleep.
    #[cfg(target_arch = "wasm32")]
    async fn send_with_retry<B: serde::Serialize, T: serde::de::DeserializeOwned>(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<&B>,
    ) -> Result<T, OpenAIError> {
        let body_value = self.prepare_body(body)?;

        for attempt in 0..=self.config.max_retries {
            let mut req = self.request(method.clone(), path);
            if let Some(ref val) = body_value {
                req = req.json(val);
            }

            let response = match req.send().await {
                Ok(resp) => resp,
                Err(e) if attempt == self.config.max_retries => {
                    return Err(OpenAIError::RequestError(e));
                }
                Err(_) => {
                    crate::runtime::sleep(crate::runtime::backoff_ms(attempt)).await;
                    continue;
                }
            };

            let status = response.status().as_u16();
            if !RETRYABLE_STATUS_CODES.contains(&status) || attempt == self.config.max_retries {
                return Self::handle_response(response).await;
            }

            crate::runtime::sleep(crate::runtime::backoff_ms(attempt)).await;
        }

        Err(OpenAIError::InvalidArgument("retry exhausted".into()))
    }

    /// Send a request with retry logic for transient errors.
    ///
    /// Fast path: first attempt avoids loop overhead and method clone.
    /// Only enters retry loop on transient errors (429, 5xx).
    #[cfg(not(target_arch = "wasm32"))]
    async fn send_with_retry<B: serde::Serialize, T: serde::de::DeserializeOwned>(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<&B>,
    ) -> Result<T, OpenAIError> {
        let body_value = self.prepare_body(body)?;

        // Fast path: first attempt — no clone, no loop
        let mut req = self.request(method.clone(), path);
        if let Some(ref val) = body_value {
            req = req.json(val);
        }

        let response = match req.send().await {
            Ok(resp) => resp,
            Err(e) if self.config.max_retries() == 0 => return Err(OpenAIError::RequestError(e)),
            Err(e) => {
                // Enter retry path
                return self.retry_loop(method, path, &body_value, e, 1).await;
            }
        };

        let status = response.status().as_u16();
        if !RETRYABLE_STATUS_CODES.contains(&status) {
            return Self::handle_response(response).await;
        }

        if self.config.max_retries() == 0 {
            return Self::handle_response(response).await;
        }

        // Retryable status on first attempt — enter retry loop
        let retry_after = response
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<f64>().ok());
        let last_error = Self::extract_error(status, response).await;
        tokio::time::sleep(Self::backoff_delay(0, retry_after)).await;
        self.retry_loop(method, path, &body_value, last_error, 1)
            .await
    }

    /// Retry loop — only called when first attempt fails with a transient error.
    #[cfg(not(target_arch = "wasm32"))]
    async fn retry_loop<T: serde::de::DeserializeOwned>(
        &self,
        method: reqwest::Method,
        path: &str,
        body_value: &Option<serde_json::Value>,
        initial_error: impl Into<OpenAIError>,
        start_attempt: u32,
    ) -> Result<T, OpenAIError> {
        let max_retries = self.config.max_retries();
        let mut last_error: OpenAIError = initial_error.into();

        for attempt in start_attempt..=max_retries {
            let mut req = self.request(method.clone(), path);
            if let Some(val) = body_value {
                req = req.json(val);
            }

            let response = match req.send().await {
                Ok(resp) => resp,
                Err(e) => {
                    last_error = OpenAIError::RequestError(e);
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

            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<f64>().ok());
            last_error = Self::extract_error(status, response).await;
            tokio::time::sleep(Self::backoff_delay(attempt, retry_after)).await;
        }

        Err(last_error)
    }

    /// Send a request with retry, returning the raw [`reqwest::Response`].
    ///
    /// Used by streaming and multipart endpoints that need retry but handle the
    /// response body themselves. Retry happens BEFORE consuming the body.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) async fn send_raw_with_retry(
        &self,
        builder: reqwest::RequestBuilder,
    ) -> Result<reqwest::Response, OpenAIError> {
        // Fast path: first attempt
        let response = match builder.try_clone() {
            Some(cloned) => match cloned.send().await {
                Ok(resp) => resp,
                Err(e) if self.config.max_retries() == 0 => {
                    return Err(OpenAIError::RequestError(e));
                }
                Err(e) => {
                    return self
                        .retry_loop_raw(builder, OpenAIError::RequestError(e), 1)
                        .await;
                }
            },
            None => {
                // Cannot clone (e.g. streaming body) — no retry possible
                return Ok(builder.send().await?);
            }
        };

        let status = response.status().as_u16();
        if !RETRYABLE_STATUS_CODES.contains(&status) {
            return Ok(response);
        }
        if self.config.max_retries() == 0 {
            return Ok(response);
        }

        let retry_after = response
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<f64>().ok());
        let last_error = Self::extract_error(status, response).await;
        tokio::time::sleep(Self::backoff_delay(0, retry_after)).await;
        self.retry_loop_raw(builder, last_error, 1).await
    }

    /// Retry loop for raw responses.
    #[cfg(not(target_arch = "wasm32"))]
    async fn retry_loop_raw(
        &self,
        builder: reqwest::RequestBuilder,
        initial_error: OpenAIError,
        start_attempt: u32,
    ) -> Result<reqwest::Response, OpenAIError> {
        let max_retries = self.config.max_retries();
        let mut last_error = initial_error;

        for attempt in start_attempt..=max_retries {
            let req = match builder.try_clone() {
                Some(cloned) => cloned,
                None => return Err(last_error),
            };

            let response = match req.send().await {
                Ok(resp) => resp,
                Err(e) => {
                    last_error = OpenAIError::RequestError(e);
                    if attempt < max_retries {
                        tokio::time::sleep(Self::backoff_delay(attempt, None)).await;
                        continue;
                    }
                    break;
                }
            };

            let status = response.status().as_u16();
            if !RETRYABLE_STATUS_CODES.contains(&status) || attempt == max_retries {
                return Ok(response);
            }

            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<f64>().ok());
            last_error = Self::extract_error(status, response).await;
            tokio::time::sleep(Self::backoff_delay(attempt, retry_after)).await;
        }

        Err(last_error)
    }

    /// Check a streaming response status and return error if non-2xx.
    pub(crate) async fn check_stream_response(
        response: reqwest::Response,
    ) -> Result<reqwest::Response, OpenAIError> {
        if response.status().is_success() {
            Ok(response)
        } else {
            Err(Self::extract_error(response.status().as_u16(), response).await)
        }
    }

    /// Calculate backoff delay: max(retry_after, 0.5 * 2^attempt) seconds.
    #[cfg(not(target_arch = "wasm32"))]
    fn backoff_delay(attempt: u32, retry_after_secs: Option<f64>) -> Duration {
        let base = crate::runtime::backoff_ms(attempt);
        match retry_after_secs {
            Some(ra) => Duration::from_secs_f64(ra.max(base.as_secs_f64())),
            None => base,
        }
    }

    /// Handle API response: check status, parse errors or deserialize body.
    ///
    /// Uses `bytes()` + `from_slice()` instead of `text()` + `from_str()`
    /// to avoid an intermediate String allocation.
    ///
    /// With `simd` feature: uses simd-json for SIMD-accelerated parsing.
    pub(crate) async fn handle_response<T: serde::de::DeserializeOwned>(
        response: reqwest::Response,
    ) -> Result<T, OpenAIError> {
        let status = response.status();
        if status.is_success() {
            let body = response.bytes().await?;
            let result = Self::deserialize_body::<T>(&body);
            match result {
                Ok(value) => Ok(value),
                Err(e) => {
                    tracing::error!(
                        error = %e,
                        body_len = body.len(),
                        body_preview = %String::from_utf8_lossy(&body[..body.len().min(500)]),
                        "failed to deserialize API response"
                    );
                    Err(e)
                }
            }
        } else {
            Err(Self::extract_error(status.as_u16(), response).await)
        }
    }

    /// Deserialize JSON body. Uses simd-json when `simd` feature is enabled.
    #[cfg(feature = "simd")]
    fn deserialize_body<T: serde::de::DeserializeOwned>(body: &[u8]) -> Result<T, OpenAIError> {
        let mut buf = body.to_vec();
        simd_json::from_slice::<T>(&mut buf)
            .map_err(|e| OpenAIError::StreamError(format!("simd-json: {e}")))
    }

    /// Deserialize JSON body (standard serde_json).
    #[cfg(not(feature = "simd"))]
    fn deserialize_body<T: serde::de::DeserializeOwned>(body: &[u8]) -> Result<T, OpenAIError> {
        serde_json::from_slice::<T>(body).map_err(OpenAIError::from)
    }

    /// Extract an OpenAIError from a failed response.
    pub(crate) async fn extract_error(status: u16, response: reqwest::Response) -> OpenAIError {
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

/// Access beta endpoints (Assistants v2, Threads, Runs, Vector Stores).
#[cfg(feature = "beta")]
pub struct Beta<'a> {
    client: &'a OpenAI,
}

#[cfg(feature = "beta")]
impl<'a> Beta<'a> {
    /// Access the Assistants resource.
    pub fn assistants(&self) -> Assistants<'_> {
        Assistants::new(self.client)
    }

    /// Access the Threads resource.
    pub fn threads(&self) -> Threads<'_> {
        Threads::new(self.client)
    }

    /// Access runs for a specific thread.
    pub fn runs(&self, thread_id: &str) -> Runs<'_> {
        Runs::new(self.client, thread_id.to_string())
    }

    /// Access the Vector Stores resource.
    pub fn vector_stores(&self) -> VectorStores<'_> {
        VectorStores::new(self.client)
    }

    /// Access the Realtime resource.
    pub fn realtime(&self) -> Realtime<'_> {
        Realtime::new(self.client)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_client() {
        let client = OpenAI::new("sk-test-key");
        assert_eq!(client.config.api_key(), "sk-test-key");
        assert_eq!(client.config.base_url(), "https://api.openai.com/v1");
    }

    #[test]
    fn test_with_config() {
        let config = ClientConfig::new("sk-test")
            .base_url("https://custom.api.com")
            .organization("org-123")
            .timeout_secs(30);
        let client = OpenAI::with_config(config);
        assert_eq!(client.config.base_url(), "https://custom.api.com");
        assert_eq!(client.config.organization(), Some("org-123"));
        assert_eq!(client.config.timeout_secs(), 30);
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

    // --- with_options() tests ---

    #[tokio::test]
    async fn test_with_options_sends_extra_headers() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/test")
            .match_header("X-Custom", "test-value")
            .with_status(200)
            .with_body(r#"{"ok":true}"#)
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        let custom = client.with_options(RequestOptions::new().header("X-Custom", "test-value"));

        #[derive(serde::Deserialize)]
        struct Resp {
            ok: bool,
        }

        let resp: Resp = custom.get("/test").await.unwrap();
        assert!(resp.ok);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_with_options_sends_query_params() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/test")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "foo".into(),
                "bar".into(),
            )]))
            .with_status(200)
            .with_body(r#"{"ok":true}"#)
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        let custom = client.with_options(RequestOptions::new().query_param("foo", "bar"));

        #[derive(serde::Deserialize)]
        struct Resp {
            ok: bool,
        }

        let resp: Resp = custom.get("/test").await.unwrap();
        assert!(resp.ok);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_extra_body_merge() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/test")
            .match_body(mockito::Matcher::Json(serde_json::json!({
                "model": "gpt-4",
                "extra_field": "injected"
            })))
            .with_status(200)
            .with_body(r#"{"id":"ok"}"#)
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        let custom = client.with_options(
            RequestOptions::new().extra_body(serde_json::json!({"extra_field": "injected"})),
        );

        #[derive(serde::Serialize)]
        struct Req {
            model: String,
        }
        #[derive(serde::Deserialize)]
        struct Resp {
            id: String,
        }

        let resp: Resp = custom
            .post(
                "/test",
                &Req {
                    model: "gpt-4".into(),
                },
            )
            .await
            .unwrap();
        assert_eq!(resp.id, "ok");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_timeout_override() {
        let mut server = mockito::Server::new_async().await;
        // Mock with a 5s delay — our timeout is 100ms, so it should fail
        let _mock = server
            .mock("GET", "/test")
            .with_status(200)
            .with_body(r#"{"ok":true}"#)
            .with_chunked_body(|_w| -> std::io::Result<()> {
                std::thread::sleep(std::time::Duration::from_secs(5));
                Ok(())
            })
            .create_async()
            .await;

        let client = OpenAI::with_config(
            ClientConfig::new("sk-test")
                .base_url(server.url())
                .max_retries(0),
        );
        let custom = client.with_options(RequestOptions::new().timeout(Duration::from_millis(100)));

        #[derive(Debug, serde::Deserialize)]
        struct Resp {
            _ok: bool,
        }

        let err = custom.get::<Resp>("/test").await.unwrap_err();
        assert!(
            matches!(err, OpenAIError::RequestError(_)),
            "expected timeout error, got: {err:?}"
        );
    }

    #[tokio::test]
    async fn test_options_merge_precedence() {
        let mut server = mockito::Server::new_async().await;
        // with_options header should override the default
        let mock = server
            .mock("GET", "/test")
            .match_header("X-A", "2")
            .with_status(200)
            .with_body(r#"{"ok":true}"#)
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        let base = client.with_options(RequestOptions::new().header("X-A", "1"));
        let custom = base.with_options(RequestOptions::new().header("X-A", "2"));

        #[derive(serde::Deserialize)]
        struct Resp {
            ok: bool,
        }

        let resp: Resp = custom.get("/test").await.unwrap();
        assert!(resp.ok);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_default_headers_and_query_on_config() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/test")
            .match_header("X-Default", "from-config")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "cfg_param".into(),
                "cfg_val".into(),
            )]))
            .with_status(200)
            .with_body(r#"{"ok":true}"#)
            .create_async()
            .await;

        let mut default_headers = reqwest::header::HeaderMap::new();
        default_headers.insert("X-Default", "from-config".parse().unwrap());

        let client = OpenAI::with_config(
            ClientConfig::new("sk-test")
                .base_url(server.url())
                .default_headers(default_headers)
                .default_query(vec![("cfg_param".into(), "cfg_val".into())]),
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
    async fn test_chained_with_options_merges() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/test")
            .match_header("X-A", "from-a")
            .match_header("X-B", "from-b")
            .with_status(200)
            .with_body(r#"{"ok":true}"#)
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        let chained = client
            .with_options(RequestOptions::new().header("X-A", "from-a"))
            .with_options(RequestOptions::new().header("X-B", "from-b"));

        #[derive(serde::Deserialize)]
        struct Resp {
            ok: bool,
        }

        let resp: Resp = chained.get("/test").await.unwrap();
        assert!(resp.ok);
        mock.assert_async().await;
    }
}
