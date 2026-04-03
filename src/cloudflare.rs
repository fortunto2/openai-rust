// Cloudflare Workers AI helpers.
//
// Workers AI is OpenAI-compatible. These helpers construct the correct
// base URL and optionally set the `x-session-affinity` header for prefix caching.
//
// OpenAI guide: <https://developers.cloudflare.com/workers-ai/configuration/open-ai-compatibility/>

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

use crate::config::ClientConfig;
use crate::error::OpenAIError;

const SESSION_AFFINITY_HEADER: &str = "x-session-affinity";

/// Options for Cloudflare AI Gateway headers.
///
/// These `cf-aig-*` headers control gateway-level behavior (caching, retries, timeouts).
/// Only used with `gateway_config()`, not with direct Workers AI.
///
/// Reference: <https://developers.cloudflare.com/ai-gateway/configuration/request-handling/>
#[derive(Debug, Clone, Default)]
pub struct GatewayOptions {
    /// Request timeout in milliseconds (e.g., 300000 = 5 min).
    pub request_timeout_ms: Option<u64>,
    /// Max retry attempts (max 5).
    pub max_attempts: Option<u8>,
    /// Retry delay in milliseconds (max 5000).
    pub retry_delay_ms: Option<u64>,
    /// Backoff strategy: "constant", "linear", or "exponential".
    pub backoff: Option<String>,
    /// Cache TTL in seconds (min 60, max ~2.6M / one month).
    pub cache_ttl_secs: Option<u64>,
    /// Skip cache for this request.
    pub skip_cache: bool,
    /// Custom cache key (overrides default request-based caching).
    pub cache_key: Option<String>,
}

fn set(headers: &mut HeaderMap, name: &'static str, value: &str) -> Result<(), OpenAIError> {
    headers.insert(
        HeaderName::from_static(name),
        HeaderValue::from_str(value)
            .map_err(|e| OpenAIError::InvalidArgument(format!("invalid header {name}: {e}")))?,
    );
    Ok(())
}

/// Build a `ClientConfig` for Cloudflare Workers AI.
///
/// ```ignore
/// let client = OpenAI::with_config(
///     cloudflare::config("account-id", "cf-token", None)?
/// );
/// // model: "@cf/meta/llama-3.3-70b-instruct-fp8-fast"
/// ```
///
/// With session affinity for prefix caching:
/// ```ignore
/// let client = OpenAI::with_config(
///     cloudflare::config("account-id", "cf-token", Some("session-123"))?
/// );
/// ```
pub fn config(
    account_id: &str,
    api_token: &str,
    session_affinity: Option<&str>,
) -> Result<ClientConfig, OpenAIError> {
    let base_url = format!("https://api.cloudflare.com/client/v4/accounts/{account_id}/ai/v1");

    let mut cfg = ClientConfig::new(api_token).base_url(base_url);

    if let Some(key) = session_affinity {
        let mut headers = HeaderMap::with_capacity(1);
        headers.insert(
            SESSION_AFFINITY_HEADER,
            HeaderValue::from_str(key).map_err(|e| {
                OpenAIError::InvalidArgument(format!("invalid session affinity key: {e}"))
            })?,
        );
        cfg = cfg.default_headers(headers);
    }

    Ok(cfg)
}

/// Build a `ClientConfig` for Cloudflare AI Gateway.
///
/// AI Gateway adds logging, caching, retries, and rate limiting on top of Workers AI.
///
/// ```ignore
/// use openai_oxide::cloudflare::{self, GatewayOptions};
///
/// let client = OpenAI::with_config(
///     cloudflare::gateway_config("account-id", "gateway-id", "cf-token", &GatewayOptions {
///         request_timeout_ms: Some(300_000),
///         cache_ttl_secs: Some(3600),
///         ..Default::default()
///     })?
/// );
/// ```
pub fn gateway_config(
    account_id: &str,
    gateway_id: &str,
    api_token: &str,
    opts: &GatewayOptions,
) -> Result<ClientConfig, OpenAIError> {
    let base_url = format!("https://gateway.ai.cloudflare.com/v1/{account_id}/{gateway_id}/openai");

    let mut headers = HeaderMap::new();

    if let Some(ms) = opts.request_timeout_ms {
        set(&mut headers, "cf-aig-request-timeout", &ms.to_string())?;
    }
    if let Some(n) = opts.max_attempts {
        set(&mut headers, "cf-aig-max-attempts", &n.to_string())?;
    }
    if let Some(ms) = opts.retry_delay_ms {
        set(&mut headers, "cf-aig-retry-delay", &ms.to_string())?;
    }
    if let Some(ref strategy) = opts.backoff {
        set(&mut headers, "cf-aig-backoff", strategy)?;
    }
    if let Some(ttl) = opts.cache_ttl_secs {
        set(&mut headers, "cf-aig-cache-ttl", &ttl.to_string())?;
    }
    if opts.skip_cache {
        set(&mut headers, "cf-aig-skip-cache", "true")?;
    }
    if let Some(ref key) = opts.cache_key {
        set(&mut headers, "cf-aig-cache-key", key)?;
    }

    let mut cfg = ClientConfig::new(api_token).base_url(base_url);

    if !headers.is_empty() {
        cfg = cfg.default_headers(headers);
    }

    Ok(cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_base_url() {
        let cfg = config("abc123", "cf-token", None).unwrap();
        assert_eq!(
            cfg.base_url,
            "https://api.cloudflare.com/client/v4/accounts/abc123/ai/v1"
        );
    }

    #[test]
    fn test_gateway_url() {
        let cfg =
            gateway_config("abc123", "my-gw", "cf-token", &GatewayOptions::default()).unwrap();
        assert_eq!(
            cfg.base_url,
            "https://gateway.ai.cloudflare.com/v1/abc123/my-gw/openai"
        );
    }

    #[test]
    fn test_gateway_headers() {
        let cfg = gateway_config(
            "abc123",
            "my-gw",
            "cf-token",
            &GatewayOptions {
                request_timeout_ms: Some(300_000),
                cache_ttl_secs: Some(3600),
                skip_cache: false,
                ..Default::default()
            },
        )
        .unwrap();
        let headers = cfg.default_headers.as_ref().unwrap();
        assert_eq!(headers.get("cf-aig-request-timeout").unwrap(), "300000");
        assert_eq!(headers.get("cf-aig-cache-ttl").unwrap(), "3600");
        assert!(headers.get("cf-aig-skip-cache").is_none());
    }

    #[test]
    fn test_session_affinity_header() {
        let cfg = config("abc123", "cf-token", Some("ses_123")).unwrap();
        let headers = cfg.default_headers.as_ref().unwrap();
        assert_eq!(headers.get(SESSION_AFFINITY_HEADER).unwrap(), "ses_123");
    }

    #[test]
    fn test_no_session_affinity() {
        let cfg = config("abc123", "cf-token", None).unwrap();
        assert!(cfg.default_headers.is_none());
    }

    #[test]
    fn test_bearer_auth() {
        let cfg = config("abc123", "cf-token", None).unwrap();
        assert_eq!(cfg.api_key, "cf-token");
    }

    #[tokio::test]
    async fn test_e2e_session_affinity() {
        use crate::client::OpenAI;
        use crate::types::chat::{ChatCompletionMessageParam, ChatCompletionRequest, UserContent};

        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/ai/v1/chat/completions")
            .match_header(SESSION_AFFINITY_HEADER, "agent-42")
            .match_header("authorization", "Bearer cf-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "id": "chatcmpl-cf-123",
                "object": "chat.completion",
                "created": 1700000000,
                "model": "@cf/meta/llama-3.3-70b-instruct-fp8-fast",
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "Hello from Cloudflare!"
                    },
                    "finish_reason": "stop"
                }],
                "usage": {
                    "prompt_tokens": 10,
                    "completion_tokens": 5,
                    "total_tokens": 15
                }
            }"#,
            )
            .create_async()
            .await;

        // Build through config() to verify it produces correct headers,
        // then rebuild with mock server URL
        let real_cfg = config("test", "cf-token", Some("agent-42")).unwrap();
        let built_headers = real_cfg.default_headers.unwrap().clone();

        let mock_cfg = ClientConfig::new("cf-token")
            .base_url(format!("{}/ai/v1", server.url()))
            .default_headers(built_headers);

        let client = OpenAI::with_config(mock_cfg);
        let request = ChatCompletionRequest::new(
            "@cf/meta/llama-3.3-70b-instruct-fp8-fast",
            vec![ChatCompletionMessageParam::User {
                content: UserContent::Text("Hello!".into()),
                name: None,
            }],
        );

        let response = client.chat().completions().create(request).await.unwrap();
        assert_eq!(
            response.choices[0].message.content.as_deref().unwrap(),
            "Hello from Cloudflare!"
        );
        mock.assert_async().await;
    }
}
