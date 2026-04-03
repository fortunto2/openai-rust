// Cloudflare Workers AI helpers.
//
// Workers AI is OpenAI-compatible. These helpers construct the correct
// base URL and optionally set the `x-session-affinity` header for prefix caching.
//
// OpenAI guide: <https://developers.cloudflare.com/workers-ai/configuration/open-ai-compatibility/>

use reqwest::header::{HeaderMap, HeaderValue};

use crate::config::ClientConfig;
use crate::error::OpenAIError;

const SESSION_AFFINITY_HEADER: &str = "x-session-affinity";

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
/// AI Gateway adds logging, caching, and rate limiting on top of Workers AI.
///
/// ```ignore
/// let client = OpenAI::with_config(
///     cloudflare::gateway_config("account-id", "gateway-id", "cf-token", None)?
/// );
/// ```
pub fn gateway_config(
    account_id: &str,
    gateway_id: &str,
    api_token: &str,
    session_affinity: Option<&str>,
) -> Result<ClientConfig, OpenAIError> {
    let base_url = format!("https://gateway.ai.cloudflare.com/v1/{account_id}/{gateway_id}/openai");

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
        let cfg = gateway_config("abc123", "my-gw", "cf-token", None).unwrap();
        assert_eq!(
            cfg.base_url,
            "https://gateway.ai.cloudflare.com/v1/abc123/my-gw/openai"
        );
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
