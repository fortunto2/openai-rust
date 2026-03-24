// Responses resource — client.responses().create() / retrieve() / delete()

use crate::client::OpenAI;
use crate::error::OpenAIError;
use crate::streaming::SseStream;
use crate::types::responses::{Response, ResponseCreateRequest, ResponseStreamEvent};

/// Metadata from a streaming FC session.
#[derive(Debug, Clone, Default)]
pub struct StreamFcMeta {
    /// Response ID (set after `response.created` event).
    pub response_id: Option<String>,
    /// Error message if the stream ended with `response.failed` or a timeout.
    pub error: Option<String>,
}

/// Handle returned by [`Responses::create_stream_fc`].
///
/// Provides early access to function calls as they complete streaming,
/// plus metadata (response_id, errors) via a separate channel.
pub struct StreamFcHandle {
    rx: tokio::sync::mpsc::Receiver<crate::types::responses::FunctionCall>,
    meta: tokio::sync::watch::Receiver<StreamFcMeta>,
}

impl StreamFcHandle {
    /// Receive the next completed function call.
    ///
    /// Returns `None` when the stream ends (either `response.completed` or error).
    pub async fn recv(&mut self) -> Option<crate::types::responses::FunctionCall> {
        self.rx.recv().await
    }

    /// Get the response ID (available after the first event).
    pub fn response_id(&self) -> Option<String> {
        self.meta.borrow().response_id.clone()
    }

    /// Check if the stream ended with an error.
    ///
    /// Call after `recv()` returns `None` to distinguish between
    /// successful completion and failure.
    pub async fn error(&mut self) -> Option<String> {
        // Wait for final meta update
        let _ = self.meta.changed().await;
        self.meta.borrow().error.clone()
    }

    /// Check current error without waiting (non-blocking).
    pub fn error_now(&self) -> Option<String> {
        self.meta.borrow().error.clone()
    }
}

/// Access the Responses API endpoints.
///
/// OpenAI guide: <https://platform.openai.com/docs/guides/responses>
/// API reference: <https://platform.openai.com/docs/api-reference/responses>
pub struct Responses<'a> {
    client: &'a OpenAI,
}

impl<'a> Responses<'a> {
    pub(crate) fn new(client: &'a OpenAI) -> Self {
        Self { client }
    }

    /// Create a response with a custom request type, returning raw JSON.
    ///
    /// Use this when you need to send fields not yet in [`ResponseCreateRequest`]
    /// or want to work with the raw API response.
    ///
    /// ```ignore
    /// use serde_json::json;
    ///
    /// let raw = client.responses().create_raw(&json!({
    ///     "model": "gpt-4o",
    ///     "input": "Hello",
    ///     "custom_field": true
    /// })).await?;
    /// println!("{}", raw["output"][0]["content"][0]["text"]);
    /// ```
    pub async fn create_raw(
        &self,
        request: &impl serde::Serialize,
    ) -> Result<serde_json::Value, OpenAIError> {
        self.client.post_json("/responses", request).await
    }

    pub async fn create_stream_raw(
        &self,
        request: &impl serde::Serialize,
    ) -> Result<crate::streaming::SseStream<serde_json::Value>, OpenAIError> {
        let builder = self
            .client
            .request(reqwest::Method::POST, "/responses")
            .header(reqwest::header::ACCEPT, "text/event-stream")
            .header(reqwest::header::CACHE_CONTROL, "no-cache")
            .json(request);

        let response = self.client.send_raw_with_retry(builder).await?;
        let response = OpenAI::check_stream_response(response).await?;
        Ok(crate::streaming::SseStream::new(response))
    }

    /// Create a response.
    ///
    /// `POST /responses`
    pub async fn create(&self, request: ResponseCreateRequest) -> Result<Response, OpenAIError> {
        self.client.post("/responses", &request).await
    }

    /// Create a response and parse the text output into a typed struct.
    ///
    /// Automatically sets `text.format` to a strict JSON schema derived
    /// from `T` using [`schemars::JsonSchema`].
    ///
    /// ```ignore
    /// #[derive(Deserialize, JsonSchema)]
    /// struct Summary { title: String, points: Vec<String> }
    ///
    /// let result = client.responses()
    ///     .parse::<Summary>(request).await?;
    /// println!("{}", result.parsed.unwrap().title);
    /// ```
    ///
    /// Requires the `structured` feature.
    #[cfg(feature = "structured")]
    pub async fn parse<T: serde::de::DeserializeOwned + schemars::JsonSchema>(
        &self,
        mut request: ResponseCreateRequest,
    ) -> Result<crate::parsing::ParsedResponse<T>, OpenAIError> {
        request.text = Some(crate::types::responses::ResponseTextConfig {
            format: Some(crate::parsing::text_format_from_type::<T>()),
            verbosity: None,
        });
        let response: Response = self.client.post("/responses", &request).await?;
        crate::parsing::parse_response(response)
    }

    /// Create a streaming response.
    ///
    /// Returns a `Stream<Item = Result<ResponseStreamEvent>>`.
    /// The `stream` field in the request is automatically set to `true`.
    pub async fn create_stream(
        &self,
        mut request: ResponseCreateRequest,
    ) -> Result<SseStream<ResponseStreamEvent>, OpenAIError> {
        request.stream = Some(true);
        let builder = self
            .client
            .request(reqwest::Method::POST, "/responses")
            .header(reqwest::header::ACCEPT, "text/event-stream")
            .header(reqwest::header::CACHE_CONTROL, "no-cache")
            .json(&request);

        let response = self.client.send_raw_with_retry(builder).await?;
        let response = OpenAI::check_stream_response(response).await?;
        Ok(SseStream::new(response))
    }

    /// Stream a response and yield function calls as soon as arguments are complete.
    ///
    /// Emits each [`FunctionCall`](crate::types::responses::FunctionCall) on the
    /// `response.function_call_arguments.done` event — typically 200-500ms before
    /// `response.completed`. Safe: the event guarantees complete, valid JSON arguments.
    ///
    /// Returns [`StreamFcHandle`] which provides:
    /// - `recv()` — next function call (None when stream ends)
    /// - `response_id()` — the response ID (available after first event)
    /// - `error()` — check if the stream ended with an error
    ///
    /// ```ignore
    /// let mut handle = client.responses()
    ///     .create_stream_fc(request)
    ///     .await?;
    ///
    /// while let Some(fc) = handle.recv().await {
    ///     let result = execute_tool(&fc.name, &fc.arguments).await;
    /// }
    /// // Check for API errors after stream ends
    /// if let Some(err) = handle.error().await {
    ///     eprintln!("API error: {err}");
    /// }
    /// ```
    pub async fn create_stream_fc(
        &self,
        request: ResponseCreateRequest,
    ) -> Result<StreamFcHandle, OpenAIError> {
        use futures_util::StreamExt;

        let mut stream = self.create_stream(request).await?;

        let (fc_tx, fc_rx) = tokio::sync::mpsc::channel(16);
        let (meta_tx, meta_rx) = tokio::sync::watch::channel(StreamFcMeta::default());

        // Spawn the stream consumer — tokio::spawn on native, spawn_local on WASM
        let spawn_future = async move {
            let mut pending_name: std::collections::HashMap<i64, String> = Default::default();
            let mut pending_call_id: std::collections::HashMap<i64, String> = Default::default();
            let mut response_id: Option<String> = None;
            let mut event_count: u32 = 0;
            const MAX_EVENTS: u32 = 10_000; // safety cap

            loop {
                event_count += 1;
                if event_count > MAX_EVENTS {
                    let _ = meta_tx.send(StreamFcMeta {
                        response_id: response_id.clone(),
                        error: Some("exceeded 10000 events limit".into()),
                    });
                    break;
                }
                let event =
                    crate::runtime::timeout(std::time::Duration::from_secs(60), stream.next())
                        .await;

                let event = match event {
                    Ok(Some(Ok(ev))) => ev,
                    Ok(Some(Err(e))) => {
                        let _ = meta_tx.send(StreamFcMeta {
                            response_id: response_id.clone(),
                            error: Some(format!("stream error: {e}")),
                        });
                        break;
                    }
                    Ok(None) => break,
                    Err(_) => {
                        let _ = meta_tx.send(StreamFcMeta {
                            response_id: response_id.clone(),
                            error: Some("timeout: no event for 60s".into()),
                        });
                        break;
                    }
                };

                match event {
                    ResponseStreamEvent::ResponseCreated { response: resp } => {
                        response_id = Some(resp.id.clone());
                        let _ = meta_tx.send(StreamFcMeta {
                            response_id: response_id.clone(),
                            error: None,
                        });
                    }
                    ResponseStreamEvent::OutputItemAdded {
                        output_index, item, ..
                    } => {
                        if item.type_ == "function_call" {
                            if let Some(name) = &item.name {
                                pending_name.insert(output_index, name.clone());
                            }
                            let cid = item.call_id.as_deref().or(item.id.as_deref()).unwrap_or("");
                            pending_call_id.insert(output_index, cid.to_string());
                        }
                    }
                    ResponseStreamEvent::FunctionCallArgumentsDone {
                        output_index,
                        arguments,
                        ..
                    } => {
                        let name = pending_name.remove(&output_index).unwrap_or_default();
                        let call_id = pending_call_id.remove(&output_index).unwrap_or_default();
                        let parsed_args = serde_json::from_str(&arguments)
                            .unwrap_or(serde_json::Value::Object(Default::default()));

                        let fc = crate::types::responses::FunctionCall {
                            call_id,
                            name,
                            arguments: parsed_args,
                        };

                        if fc_tx.send(fc).await.is_err() {
                            break; // receiver dropped
                        }
                    }
                    ResponseStreamEvent::ResponseFailed { response: resp } => {
                        let msg = resp
                            .error
                            .as_ref()
                            .map(|e| e.message.clone())
                            .unwrap_or_else(|| "response.failed".into());
                        let _ = meta_tx.send(StreamFcMeta {
                            response_id: response_id.clone(),
                            error: Some(msg),
                        });
                        break;
                    }
                    ResponseStreamEvent::ResponseCompleted { .. } => {
                        let _ = meta_tx.send(StreamFcMeta {
                            response_id: response_id.clone(),
                            error: None,
                        });
                        break;
                    }
                    _ => {}
                }
            }
        };

        crate::runtime::spawn(spawn_future);

        Ok(StreamFcHandle {
            rx: fc_rx,
            meta: meta_rx,
        })
    }

    /// Retrieve a response by ID.
    ///
    /// `GET /responses/{response_id}`
    pub async fn retrieve(&self, response_id: &str) -> Result<Response, OpenAIError> {
        self.client.get(&format!("/responses/{response_id}")).await
    }

    /// Cancel a background response.
    ///
    /// `POST /responses/{response_id}/cancel`
    pub async fn cancel(&self, response_id: &str) -> Result<Response, OpenAIError> {
        self.client
            .post_empty(&format!("/responses/{response_id}/cancel"))
            .await
    }

    /// List input items for a response.
    ///
    /// `GET /responses/{response_id}/input_items`
    pub async fn input_items(&self, response_id: &str) -> Result<serde_json::Value, OpenAIError> {
        self.client
            .get(&format!("/responses/{response_id}/input_items"))
            .await
    }

    /// Count input tokens for a request without creating a response.
    ///
    /// `POST /responses/input_tokens`
    pub async fn count_tokens(
        &self,
        request: &ResponseCreateRequest,
    ) -> Result<serde_json::Value, OpenAIError> {
        self.client.post("/responses/input_tokens", request).await
    }

    /// Compact a conversation — reduces token count for long-running sessions.
    ///
    /// `POST /responses/compact`
    ///
    /// See [conversation state guide](https://platform.openai.com/docs/guides/conversation-state#managing-the-context-window).
    pub async fn compact(
        &self,
        body: &impl serde::Serialize,
    ) -> Result<serde_json::Value, OpenAIError> {
        self.client.post("/responses/compact", body).await
    }

    /// Delete a response.
    ///
    /// `DELETE /responses/{response_id}`
    pub async fn delete(&self, response_id: &str) -> Result<(), OpenAIError> {
        let response = self
            .client
            .request(
                reqwest::Method::DELETE,
                &format!("/responses/{response_id}"),
            )
            .send()
            .await?;

        OpenAI::check_stream_response(response).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::OpenAI;
    use crate::config::ClientConfig;
    use crate::types::responses::ResponseCreateRequest;

    const RESPONSE_JSON: &str = r#"{
        "id": "resp-abc123",
        "object": "response",
        "created_at": 1677610602.0,
        "model": "gpt-4o",
        "output": [{
            "type": "message",
            "id": "msg-abc123",
            "role": "assistant",
            "status": "completed",
            "content": [{
                "type": "output_text",
                "text": "Hello!",
                "annotations": []
            }]
        }],
        "status": "completed",
        "usage": {
            "input_tokens": 10,
            "output_tokens": 2,
            "total_tokens": 12
        }
    }"#;

    #[tokio::test]
    async fn test_responses_create() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/responses")
            .match_header("authorization", "Bearer sk-test")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(RESPONSE_JSON)
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        let mut request = ResponseCreateRequest::new("gpt-4o");
        request.input = Some("Hello".into());

        let response = client.responses().create(request).await.unwrap();
        assert_eq!(response.id, "resp-abc123");
        assert_eq!(response.output_text(), "Hello!");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_responses_create_raw() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/responses")
            .match_header("authorization", "Bearer sk-test")
            .match_body(mockito::Matcher::Json(serde_json::json!({
                "model": "gpt-4o",
                "input": "Hello",
                "custom_field": "extra"
            })))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":"resp-raw","object":"response","custom_resp":99}"#)
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));

        let raw = client
            .responses()
            .create_raw(&serde_json::json!({
                "model": "gpt-4o",
                "input": "Hello",
                "custom_field": "extra"
            }))
            .await
            .unwrap();

        assert_eq!(raw["id"], "resp-raw");
        assert_eq!(raw["custom_resp"], 99);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_responses_retrieve() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/responses/resp-abc123")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(RESPONSE_JSON)
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        let response = client.responses().retrieve("resp-abc123").await.unwrap();
        assert_eq!(response.id, "resp-abc123");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_responses_create_with_tools() {
        use crate::types::responses::{Reasoning, ResponseTool};

        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/responses")
            .match_header("authorization", "Bearer sk-test")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(RESPONSE_JSON)
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        let mut request = ResponseCreateRequest::new("gpt-4o");
        request.input = Some("Search for Rust".into());
        request.tools = Some(vec![ResponseTool::WebSearch {
            search_context_size: Some("medium".into()),
            user_location: None,
        }]);
        request.reasoning = Some(Reasoning {
            effort: Some("high".into()),
            summary: None,
        });
        request.truncation = Some("auto".into());

        let response = client.responses().create(request).await.unwrap();
        assert_eq!(response.id, "resp-abc123");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_responses_delete() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("DELETE", "/responses/resp-abc123")
            .with_status(200)
            .with_body("")
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        client.responses().delete("resp-abc123").await.unwrap();
        mock.assert_async().await;
    }
}
