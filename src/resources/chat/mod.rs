// Chat resource — client.chat().completions().create()

use crate::client::OpenAI;
use crate::error::OpenAIError;
use crate::streaming::SseStream;
use crate::types::chat::{ChatCompletionChunk, ChatCompletionRequest, ChatCompletionResponse};

/// Access chat-related endpoints.
///
/// OpenAI guide: <https://platform.openai.com/docs/guides/chat-completions>
/// API reference: <https://platform.openai.com/docs/api-reference/chat>
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
        let builder = self
            .client
            .request(reqwest::Method::POST, "/chat/completions")
            .header(reqwest::header::ACCEPT, "text/event-stream")
            .header(reqwest::header::CACHE_CONTROL, "no-cache")
            .json(request);

        let response = self.client.send_raw_with_retry(builder).await?;
        let response = OpenAI::check_stream_response(response).await?;
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

    /// Create a chat completion and parse the response into a typed struct.
    ///
    /// Automatically sets `response_format` to a strict JSON schema derived
    /// from `T` using [`schemars::JsonSchema`]. The response content is
    /// deserialized into `T` and returned in [`ParsedChatCompletion::parsed`].
    ///
    /// ```ignore
    /// #[derive(Deserialize, JsonSchema)]
    /// struct Answer { text: String, confidence: f64 }
    ///
    /// let result = client.chat().completions()
    ///     .parse::<Answer>(request).await?;
    /// println!("{}", result.parsed.unwrap().text);
    /// ```
    ///
    /// Requires the `structured` feature.
    #[cfg(feature = "structured")]
    pub async fn parse<T: serde::de::DeserializeOwned + schemars::JsonSchema>(
        &self,
        mut request: ChatCompletionRequest,
    ) -> Result<crate::parsing::ParsedChatCompletion<T>, OpenAIError> {
        request.response_format = Some(crate::parsing::response_format_from_type::<T>());
        Self::prepare_reasoning_request(&mut request);
        let response: ChatCompletionResponse =
            self.client.post("/chat/completions", &request).await?;
        crate::parsing::parse_completion(response)
    }

    /// Create a streaming chat completion with high-level typed events.
    ///
    /// Returns a [`ChatCompletionStream`](crate::stream_helpers::ChatCompletionStream)
    /// that yields [`ChatStreamEvent`](crate::stream_helpers::ChatStreamEvent) with
    /// automatic text/tool-call accumulation.
    ///
    /// Use `.get_final_completion()` to consume the stream and get the
    /// assembled [`ChatCompletionResponse`].
    pub async fn create_stream_helper(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<crate::stream_helpers::ChatCompletionStream, OpenAIError> {
        let stream = self.create_stream(request).await?;
        Ok(crate::stream_helpers::ChatCompletionStream::new(stream))
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
        let builder = self
            .client
            .request(reqwest::Method::POST, "/chat/completions")
            .header(reqwest::header::ACCEPT, "text/event-stream")
            .header(reqwest::header::CACHE_CONTROL, "no-cache")
            .json(&request);

        let response = self.client.send_raw_with_retry(builder).await?;
        let response = OpenAI::check_stream_response(response).await?;
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

    /// Retrieve a stored chat completion by ID.
    ///
    /// `GET /chat/completions/{completion_id}`
    ///
    /// Requires the completion to have been created with `store: true`.
    pub async fn retrieve(
        &self,
        completion_id: &str,
    ) -> Result<ChatCompletionResponse, OpenAIError> {
        self.client
            .get(&format!("/chat/completions/{completion_id}"))
            .await
    }

    /// List stored chat completions.
    ///
    /// `GET /chat/completions`
    pub async fn list_stored(
        &self,
        params: &[(String, String)],
    ) -> Result<serde_json::Value, OpenAIError> {
        self.client
            .get_with_query("/chat/completions", params)
            .await
    }

    /// Delete a stored chat completion.
    ///
    /// `DELETE /chat/completions/{completion_id}`
    pub async fn delete(&self, completion_id: &str) -> Result<serde_json::Value, OpenAIError> {
        self.client
            .delete(&format!("/chat/completions/{completion_id}"))
            .await
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

    #[tokio::test]
    async fn test_chat_stream_retries_on_429() {
        use futures_util::StreamExt;

        let mut server = mockito::Server::new_async().await;

        // First request: 429
        let _mock_429 = server
            .mock("POST", "/chat/completions")
            .with_status(429)
            .with_body(r#"{"error":{"message":"Rate limit","type":"rate_limit","param":null,"code":null}}"#)
            .expect(1)
            .create_async()
            .await;

        // Second request: 200 with SSE
        let _mock_ok = server
            .mock("POST", "/chat/completions")
            .with_status(200)
            .with_header("content-type", "text/event-stream")
            .with_body("data: {\"id\":\"c1\",\"object\":\"chat.completion.chunk\",\"created\":1,\"model\":\"gpt-4o\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"Hi\"},\"finish_reason\":null}]}\n\ndata: [DONE]\n\n")
            .expect(1)
            .create_async()
            .await;

        let client = OpenAI::with_config(
            ClientConfig::new("sk-test")
                .base_url(server.url())
                .max_retries(2),
        );

        let request = ChatCompletionRequest::new(
            "gpt-4o",
            vec![ChatCompletionMessageParam::User {
                content: UserContent::Text("Hi".into()),
                name: None,
            }],
        );

        let stream = client
            .chat()
            .completions()
            .create_stream(request)
            .await
            .unwrap();

        let chunks: Vec<_> = stream
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .filter_map(|r| r.ok())
            .collect();

        assert!(!chunks.is_empty());
        assert_eq!(chunks[0].choices[0].delta.content.as_deref(), Some("Hi"));

        _mock_429.assert_async().await;
        _mock_ok.assert_async().await;
    }
}
