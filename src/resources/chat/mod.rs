// Chat resource — client.chat().completions().create()

use crate::client::OpenAI;
use crate::error::OpenAIError;
use crate::streaming::SseStream;
use crate::types::chat::{ChatCompletionChunk, ChatCompletionRequest, ChatCompletionResponse};

/// Access chat-related endpoints.
pub struct Chat<'a> {
    client: &'a OpenAI,
}

impl<'a> Chat<'a> {
    pub(crate) fn new(client: &'a OpenAI) -> Self {
        Self { client }
    }

    /// Access the completions sub-resource.
    pub fn completions(&self) -> Completions<'_> {
        Completions {
            client: self.client,
        }
    }
}

/// Chat completions endpoint.
pub struct Completions<'a> {
    client: &'a OpenAI,
}

impl<'a> Completions<'a> {
    pub async fn create_stream_raw(
        &self,
        request: &impl serde::Serialize,
    ) -> Result<crate::streaming::SseStream<serde_json::Value>, OpenAIError> {
        let response = self
            .client
            .request(reqwest::Method::POST, "/chat/completions")
            .header(reqwest::header::ACCEPT, "text/event-stream")
            .header(reqwest::header::CACHE_CONTROL, "no-cache")
            .json(request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let status_code = status.as_u16();
            let body = response.text().await.unwrap_or_default();
            if let Ok(error_resp) = serde_json::from_str::<crate::error::ErrorResponse>(&body) {
                return Err(OpenAIError::ApiError {
                    status: status_code,
                    message: error_resp.error.message,
                    type_: error_resp.error.type_,
                    code: error_resp.error.code,
                });
            }
            return Err(OpenAIError::ApiError {
                status: status_code,
                message: body,
                type_: None,
                code: None,
            });
        }

        Ok(crate::streaming::SseStream::new(response))
    }

    /// Create a chat completion with a custom request type, returning raw JSON.
    ///
    /// Use this when you need to send fields not yet in [`ChatCompletionRequest`]
    /// or want to work with the raw API response.
    ///
    /// ```ignore
    /// use serde_json::json;
    ///
    /// let raw = client.chat().completions().create_raw(&json!({
    ///     "model": "gpt-4o",
    ///     "messages": [{"role": "user", "content": "Hi"}],
    ///     "custom_field": true
    /// })).await?;
    /// println!("{}", raw["choices"][0]["message"]["content"]);
    /// ```
    pub async fn create_raw(
        &self,
        request: &impl serde::Serialize,
    ) -> Result<serde_json::Value, crate::error::OpenAIError> {
        self.client.post_json("/chat/completions", request).await
    }

    /// Create a chat completion.
    ///
    /// `POST /chat/completions`
    pub async fn create(
        &self,
        mut request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, OpenAIError> {
        Self::prepare_reasoning_request(&mut request);
        self.client.post("/chat/completions", &request).await
    }

    /// Create a streaming chat completion.
    ///
    /// Returns a `Stream<Item = Result<ChatCompletionChunk>>`.
    /// The `stream` field in the request is automatically set to `true`.
    pub async fn create_stream(
        &self,
        mut request: ChatCompletionRequest,
    ) -> Result<SseStream<ChatCompletionChunk>, OpenAIError> {
        Self::prepare_reasoning_request(&mut request);
        request.stream = Some(true);
        let response = self
            .client
            .request(reqwest::Method::POST, "/chat/completions")
            .header(reqwest::header::ACCEPT, "text/event-stream")
            .header(reqwest::header::CACHE_CONTROL, "no-cache")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let status_code = status.as_u16();
            let body = response.text().await.unwrap_or_default();
            if let Ok(error_resp) = serde_json::from_str::<crate::error::ErrorResponse>(&body) {
                return Err(OpenAIError::ApiError {
                    status: status_code,
                    message: error_resp.error.message,
                    type_: error_resp.error.type_,
                    code: error_resp.error.code,
                });
            }
            return Err(OpenAIError::ApiError {
                status: status_code,
                message: body,
                type_: None,
                code: None,
            });
        }

        Ok(SseStream::new(response))
    }

    /// Automatically aligns parameters for O1/O3 reasoning models to prevent API errors.
    fn prepare_reasoning_request(request: &mut ChatCompletionRequest) {
        if request.model.starts_with("o1") || request.model.starts_with("o3") {
            // Reasoning models crash if temperature or other generation parameters are passed
            if request.temperature.is_some() {
                tracing::warn!(
                    "temperature is not supported for reasoning models. Dropping parameter."
                );
                request.temperature = None;
            }
            if request.top_p.is_some() {
                tracing::warn!("top_p is not supported for reasoning models. Dropping parameter.");
                request.top_p = None;
            }
            if request.presence_penalty.is_some() {
                tracing::warn!(
                    "presence_penalty is not supported for reasoning models. Dropping parameter."
                );
                request.presence_penalty = None;
            }
            if request.frequency_penalty.is_some() {
                tracing::warn!(
                    "frequency_penalty is not supported for reasoning models. Dropping parameter."
                );
                request.frequency_penalty = None;
            }

            // Map max_tokens -> max_completion_tokens
            if request.max_tokens.is_some() && request.max_completion_tokens.is_none() {
                tracing::debug!("Mapping max_tokens to max_completion_tokens for reasoning model");
                request.max_completion_tokens = request.max_tokens;
                request.max_tokens = None;
            }

            // Change system messages to developer messages
            for msg in request.messages.iter_mut() {
                if let crate::types::chat::ChatCompletionMessageParam::System { content, name } =
                    msg
                {
                    tracing::debug!(
                        "Converting system message to developer message for reasoning model"
                    );
                    *msg = crate::types::chat::ChatCompletionMessageParam::Developer {
                        content: content.clone(),
                        name: name.clone(),
                    };
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::OpenAI;
    use crate::config::ClientConfig;
    use crate::types::chat::{ChatCompletionMessageParam, ChatCompletionRequest, UserContent};

    #[tokio::test]
    async fn test_chat_completions_create() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/chat/completions")
            .match_header("authorization", "Bearer sk-test")
            .match_header("content-type", "application/json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "id": "chatcmpl-abc123",
                    "object": "chat.completion",
                    "created": 1677858242,
                    "model": "gpt-4o-mini",
                    "choices": [{
                        "index": 0,
                        "message": {
                            "role": "assistant",
                            "content": "Hello! How can I help?"
                        },
                        "logprobs": null,
                        "finish_reason": "stop"
                    }],
                    "usage": {
                        "prompt_tokens": 10,
                        "completion_tokens": 6,
                        "total_tokens": 16
                    }
                }"#,
            )
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));

        let request = ChatCompletionRequest::new(
            "gpt-4o-mini",
            vec![ChatCompletionMessageParam::User {
                content: UserContent::Text("Hello".into()),
                name: None,
            }],
        );

        let response = client.chat().completions().create(request).await.unwrap();
        assert_eq!(response.id, "chatcmpl-abc123");
        assert_eq!(
            response.choices[0].finish_reason,
            crate::types::common::FinishReason::Stop
        );
        assert_eq!(
            response.choices[0].message.content.as_deref(),
            Some("Hello! How can I help?")
        );
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_chat_completions_create_raw() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/chat/completions")
            .match_header("authorization", "Bearer sk-test")
            .match_body(mockito::Matcher::Json(serde_json::json!({
                "model": "gpt-4o",
                "messages": [{"role": "user", "content": "Hi"}],
                "custom_field": true
            })))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":"chatcmpl-raw","object":"chat.completion","custom_resp":42}"#)
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));

        let raw = client
            .chat()
            .completions()
            .create_raw(&serde_json::json!({
                "model": "gpt-4o",
                "messages": [{"role": "user", "content": "Hi"}],
                "custom_field": true
            }))
            .await
            .unwrap();

        assert_eq!(raw["id"], "chatcmpl-raw");
        assert_eq!(raw["custom_resp"], 42);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_chat_completions_api_error() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("POST", "/chat/completions")
            .with_status(401)
            .with_body(
                r#"{"error":{"message":"Incorrect API key provided","type":"invalid_request_error","param":null,"code":"invalid_api_key"}}"#,
            )
            .create_async()
            .await;

        let client = OpenAI::with_config(
            ClientConfig::new("sk-bad")
                .base_url(server.url())
                .max_retries(0),
        );

        let request = ChatCompletionRequest::new(
            "gpt-4o",
            vec![ChatCompletionMessageParam::User {
                content: UserContent::Text("Hi".into()),
                name: None,
            }],
        );

        let err = client
            .chat()
            .completions()
            .create(request)
            .await
            .unwrap_err();
        match err {
            crate::error::OpenAIError::ApiError {
                status, message, ..
            } => {
                assert_eq!(status, 401);
                assert!(message.contains("API key"));
            }
            other => panic!("expected ApiError, got: {other:?}"),
        }
    }
}
