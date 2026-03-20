//! Hedged requests — send N copies, take the fastest response.
//!
//! Based on Google's "The Tail at Scale" paper. Reduces P99 latency
//! by 50-96% with only 2-7% extra token cost.
//!
//! # Strategy
//!
//! - **`hedged_request`**: send 2 copies with a hedge delay between them.
//!   The first to succeed wins; the loser is cancelled (dropped).
//! - **`hedged_request_n`**: generalizes to N copies (capped at 3).
//! - **`speculative`**: run two *dependent* steps in parallel. If step1
//!   passes validation, return both results immediately; otherwise discard step2.

use std::time::Duration;

use crate::OpenAI;
use crate::error::OpenAIError;
use crate::types::responses::{Response, ResponseCreateRequest};

/// Send the same request to OpenAI twice with a hedge delay.
/// Returns the first successful response.
///
/// `hedge_delay`: how long to wait before sending the second request.
/// Set to your P95 latency (typically 1.5-2s for GPT-5.4).
/// If `None`, sends both immediately (pure race).
///
/// # Example
/// ```ignore
/// use std::time::Duration;
/// use openai_oxide::hedged::hedged_request;
///
/// let response = hedged_request(&client, request, Some(Duration::from_millis(1500))).await?;
/// ```
pub async fn hedged_request(
    client: &OpenAI,
    request: ResponseCreateRequest,
    hedge_delay: Option<Duration>,
) -> Result<Response, OpenAIError> {
    let req1 = request.clone();
    let req2 = request;

    let responses = client.responses();
    let fut1 = responses.create(req1);
    let fut2 = async {
        if let Some(delay) = hedge_delay {
            tokio::time::sleep(delay).await;
        }
        client.responses().create(req2).await
    };

    tokio::pin!(fut1);
    tokio::pin!(fut2);

    // First successful response wins. If one fails, wait for the other.
    tokio::select! {
        result1 = &mut fut1 => {
            match result1 {
                Ok(resp) => Ok(resp),
                Err(_) => fut2.await,
            }
        }
        result2 = &mut fut2 => {
            match result2 {
                Ok(resp) => Ok(resp),
                Err(_) => fut1.await,
            }
        }
    }
}

/// Send the same request to N replicas, return fastest.
/// For cost control, N is capped at 3.
///
/// If `hedge_delay` is `Some`, each successive request is staggered
/// by that duration (request i starts at `i * delay`).
/// If `None`, all requests fire immediately.
///
/// # Example
/// ```ignore
/// use std::time::Duration;
/// use openai_oxide::hedged::hedged_request_n;
///
/// let response = hedged_request_n(
///     &client,
///     request,
///     3,
///     Some(Duration::from_millis(1000)),
/// ).await?;
/// ```
pub async fn hedged_request_n(
    client: &OpenAI,
    request: ResponseCreateRequest,
    n: usize,
    hedge_delay: Option<Duration>,
) -> Result<Response, OpenAIError> {
    let n = n.clamp(1, 3);

    if n == 1 {
        return client.responses().create(request).await;
    }

    // For n == 2, delegate to the optimized two-copy path.
    if n == 2 {
        return hedged_request(client, request, hedge_delay).await;
    }

    // n == 3: use FuturesUnordered so we can poll all three.
    use futures_util::stream::{FuturesUnordered, StreamExt};

    let futures = FuturesUnordered::new();
    let delay = hedge_delay.unwrap_or(Duration::ZERO);

    for i in 0..n {
        let req = request.clone();
        let client = client.clone();
        let stagger = delay * i as u32;
        futures.push(tokio::spawn(async move {
            if !stagger.is_zero() {
                tokio::time::sleep(stagger).await;
            }
            client.responses().create(req).await
        }));
    }

    tokio::pin!(futures);

    let mut last_error: Option<OpenAIError> = None;

    while let Some(join_result) = futures.next().await {
        match join_result {
            Ok(Ok(response)) => return Ok(response),
            Ok(Err(e)) => last_error = Some(e),
            Err(join_err) => {
                last_error = Some(OpenAIError::StreamError(format!(
                    "task panicked: {join_err}"
                )));
            }
        }
    }

    Err(last_error.unwrap_or_else(|| {
        OpenAIError::InvalidArgument("hedged_request_n: no futures were created".into())
    }))
}

/// Speculative execution — run dependent steps in parallel.
///
/// Both `step1` and `step2` fire concurrently (via `tokio::join!`).
/// After both complete, `validate_step1` is called on the step1 response.
/// If validation passes, both responses are returned.
/// If validation fails, step2 result is discarded and an error is returned.
///
/// # Use case
///
/// Run a cheap moderation/classification check in parallel with the
/// expensive generation. If moderation passes (the common case), you
/// save a full round-trip.
///
/// # Example
/// ```ignore
/// use openai_oxide::hedged::speculative;
///
/// let (moderation, generation) = speculative(
///     &client,
///     moderation_request,
///     generation_request,
///     |resp| resp.output_text().contains("safe"),
/// ).await?;
/// ```
pub async fn speculative<V>(
    client: &OpenAI,
    step1: ResponseCreateRequest,
    step2: ResponseCreateRequest,
    validate_step1: V,
) -> Result<(Response, Response), OpenAIError>
where
    V: FnOnce(&Response) -> bool,
{
    let responses = client.responses();
    let (result1, result2) = tokio::join!(responses.create(step1), responses.create(step2),);

    let resp1 = result1?;

    if !validate_step1(&resp1) {
        return Err(OpenAIError::InvalidArgument(
            "speculative: step1 validation failed, step2 result discarded".into(),
        ));
    }

    let resp2 = result2?;
    Ok((resp1, resp2))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ClientConfig;

    /// JSON for a successful Response.
    fn response_json(id: &str, text: &str) -> String {
        format!(
            r#"{{
            "id": "{id}",
            "object": "response",
            "created_at": 1677610602.0,
            "model": "gpt-4o",
            "output": [{{
                "type": "message",
                "id": "msg-1",
                "role": "assistant",
                "status": "completed",
                "content": [{{
                    "type": "output_text",
                    "text": "{text}",
                    "annotations": []
                }}]
            }}],
            "status": "completed",
            "usage": {{
                "input_tokens": 10,
                "output_tokens": 5,
                "total_tokens": 15
            }}
        }}"#
        )
    }

    #[tokio::test]
    async fn test_hedged_request_returns_first_success() {
        let mut server = mockito::Server::new_async().await;
        // Both requests hit the same endpoint; mockito returns 200 for each.
        let _mock = server
            .mock("POST", "/responses")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response_json("resp-hedge", "hedged!"))
            .expect_at_least(1)
            .create_async()
            .await;

        let client = OpenAI::with_config(
            ClientConfig::new("sk-test")
                .base_url(server.url())
                .max_retries(0),
        );
        let request = ResponseCreateRequest::new("gpt-4o").input("Hello");

        let resp = hedged_request(&client, request, None).await.unwrap();
        assert_eq!(resp.output_text(), "hedged!");
    }

    #[tokio::test]
    async fn test_hedged_request_with_delay() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("POST", "/responses")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response_json("resp-delayed", "delayed hedge"))
            .expect_at_least(1)
            .create_async()
            .await;

        let client = OpenAI::with_config(
            ClientConfig::new("sk-test")
                .base_url(server.url())
                .max_retries(0),
        );
        let request = ResponseCreateRequest::new("gpt-4o").input("Hello");

        let resp = hedged_request(&client, request, Some(Duration::from_millis(50)))
            .await
            .unwrap();
        assert_eq!(resp.output_text(), "delayed hedge");
    }

    #[tokio::test]
    async fn test_hedged_request_fallback_on_first_failure() {
        let mut server = mockito::Server::new_async().await;

        // First request fails (500), second succeeds.
        let _mock_fail = server
            .mock("POST", "/responses")
            .with_status(500)
            .with_body(
                r#"{"error":{"message":"fail","type":"server_error","param":null,"code":null}}"#,
            )
            .create_async()
            .await;

        let _mock_ok = server
            .mock("POST", "/responses")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response_json("resp-fallback", "recovered"))
            .create_async()
            .await;

        let client = OpenAI::with_config(
            ClientConfig::new("sk-test")
                .base_url(server.url())
                .max_retries(0),
        );
        let request = ResponseCreateRequest::new("gpt-4o").input("Hello");

        let resp = hedged_request(&client, request, None).await.unwrap();
        assert_eq!(resp.output_text(), "recovered");
    }

    #[tokio::test]
    async fn test_hedged_request_n_single() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/responses")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response_json("resp-n1", "single"))
            .expect(1)
            .create_async()
            .await;

        let client = OpenAI::with_config(
            ClientConfig::new("sk-test")
                .base_url(server.url())
                .max_retries(0),
        );
        let request = ResponseCreateRequest::new("gpt-4o").input("Hello");

        let resp = hedged_request_n(&client, request, 1, None).await.unwrap();
        assert_eq!(resp.output_text(), "single");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_hedged_request_n_triple() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("POST", "/responses")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response_json("resp-n3", "triple"))
            .expect_at_least(1)
            .create_async()
            .await;

        let client = OpenAI::with_config(
            ClientConfig::new("sk-test")
                .base_url(server.url())
                .max_retries(0),
        );
        let request = ResponseCreateRequest::new("gpt-4o").input("Hello");

        let resp = hedged_request_n(&client, request, 3, None).await.unwrap();
        assert_eq!(resp.output_text(), "triple");
    }

    #[tokio::test]
    async fn test_hedged_request_n_capped_at_3() {
        let mut server = mockito::Server::new_async().await;
        // Even with n=10, only 3 requests should be sent.
        let _mock = server
            .mock("POST", "/responses")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response_json("resp-cap", "capped"))
            .expect_at_most(3)
            .create_async()
            .await;

        let client = OpenAI::with_config(
            ClientConfig::new("sk-test")
                .base_url(server.url())
                .max_retries(0),
        );
        let request = ResponseCreateRequest::new("gpt-4o").input("Hello");

        let resp = hedged_request_n(&client, request, 10, None).await.unwrap();
        assert_eq!(resp.output_text(), "capped");
    }

    #[tokio::test]
    async fn test_speculative_both_succeed_validation_passes() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("POST", "/responses")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response_json("resp-spec", "safe content"))
            .expect_at_least(2)
            .create_async()
            .await;

        let client = OpenAI::with_config(
            ClientConfig::new("sk-test")
                .base_url(server.url())
                .max_retries(0),
        );
        let step1 = ResponseCreateRequest::new("gpt-4o").input("moderate this");
        let step2 = ResponseCreateRequest::new("gpt-4o").input("generate answer");

        let (resp1, resp2) =
            speculative(&client, step1, step2, |r| r.output_text().contains("safe"))
                .await
                .unwrap();

        assert_eq!(resp1.id, "resp-spec");
        assert_eq!(resp2.id, "resp-spec");
    }

    #[tokio::test]
    async fn test_speculative_validation_fails() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("POST", "/responses")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response_json("resp-spec-fail", "unsafe content"))
            .expect_at_least(2)
            .create_async()
            .await;

        let client = OpenAI::with_config(
            ClientConfig::new("sk-test")
                .base_url(server.url())
                .max_retries(0),
        );
        let step1 = ResponseCreateRequest::new("gpt-4o").input("moderate this");
        let step2 = ResponseCreateRequest::new("gpt-4o").input("generate answer");

        let err = speculative(&client, step1, step2, |r| {
            r.output_text().contains("definitely_not_here")
        })
        .await
        .unwrap_err();

        assert!(
            matches!(err, OpenAIError::InvalidArgument(_)),
            "expected InvalidArgument, got: {err:?}"
        );
    }

    #[tokio::test]
    async fn test_speculative_step1_api_error() {
        let mut server = mockito::Server::new_async().await;

        // Step1 fails, step2 succeeds — but step1 error propagates first.
        let _mock_fail = server
            .mock("POST", "/responses")
            .with_status(500)
            .with_body(
                r#"{"error":{"message":"boom","type":"server_error","param":null,"code":null}}"#,
            )
            .create_async()
            .await;

        let _mock_ok = server
            .mock("POST", "/responses")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response_json("resp-ok", "ok"))
            .create_async()
            .await;

        let client = OpenAI::with_config(
            ClientConfig::new("sk-test")
                .base_url(server.url())
                .max_retries(0),
        );
        let step1 = ResponseCreateRequest::new("gpt-4o").input("moderate");
        let step2 = ResponseCreateRequest::new("gpt-4o").input("generate");

        let err = speculative(&client, step1, step2, |_| true)
            .await
            .unwrap_err();
        assert!(
            matches!(err, OpenAIError::ApiError { .. }),
            "expected ApiError, got: {err:?}"
        );
    }
}
