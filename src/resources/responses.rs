// Responses resource — client.responses().create() / retrieve() / delete()

use crate::client::OpenAI;
use crate::error::OpenAIError;
use crate::streaming::SseStream;
use crate::types::responses::{Response, ResponseCreateRequest, ResponseStreamEvent};

/// Access the Responses API endpoints.
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

    /// Create a response.
    ///
    /// `POST /responses`
    pub async fn create(&self, request: ResponseCreateRequest) -> Result<Response, OpenAIError> {
        self.client.post("/responses", &request).await
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
        let response = self
            .client
            .request(reqwest::Method::POST, "/responses")
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

    /// Stream a response and yield function calls as soon as their arguments are complete.
    ///
    /// Returns a channel receiver that emits [`FunctionCall`](crate::types::responses::FunctionCall)
    /// items as each one finishes streaming (on `response.function_call_arguments.done`),
    /// WITHOUT waiting for `response.completed`.
    ///
    /// This lets you start executing tools ~200-500ms earlier per call in agent loops.
    ///
    /// Also returns the response_id (available after `response.created`).
    ///
    /// ```ignore
    /// let (mut rx, response_id) = client.responses()
    ///     .create_stream_fc(request)
    ///     .await?;
    ///
    /// while let Some(fc) = rx.recv().await {
    ///     // Start executing tool immediately — don't wait for response.completed
    ///     let result = execute_tool(&fc.name, &fc.arguments).await;
    /// }
    /// ```
    pub async fn create_stream_fc(
        &self,
        request: ResponseCreateRequest,
    ) -> Result<
        (
            tokio::sync::mpsc::Receiver<crate::types::responses::FunctionCall>,
            tokio::sync::watch::Receiver<Option<String>>,
        ),
        OpenAIError,
    > {
        use futures_util::StreamExt;

        let mut stream = self.create_stream(request).await?;

        let (fc_tx, fc_rx) = tokio::sync::mpsc::channel(16);
        let (id_tx, id_rx) = tokio::sync::watch::channel(None);

        // Track in-flight function calls by output_index
        tokio::spawn(async move {
            let mut pending_name: std::collections::HashMap<i64, String> = Default::default();
            let mut pending_call_id: std::collections::HashMap<i64, String> = Default::default();

            while let Some(event) = stream.next().await {
                let ev = match event {
                    Ok(ev) => ev,
                    Err(_) => break,
                };

                match ev.type_.as_str() {
                    "response.created" => {
                        if let Some(id) = ev
                            .data
                            .get("response")
                            .and_then(|r| r.get("id"))
                            .and_then(|id| id.as_str())
                        {
                            let _ = id_tx.send(Some(id.to_string()));
                        }
                    }
                    "response.output_item.added" => {
                        // Track new function_call items
                        if let Some(item) = ev.data.get("item")
                            && item.get("type").and_then(|t| t.as_str()) == Some("function_call")
                        {
                            let idx = ev
                                .data
                                .get("output_index")
                                .and_then(|i| i.as_i64())
                                .unwrap_or(-1);
                            if let Some(name) = item.get("name").and_then(|n| n.as_str()) {
                                pending_name.insert(idx, name.to_string());
                            }
                            if let Some(cid) = item.get("call_id").and_then(|c| c.as_str()) {
                                pending_call_id.insert(idx, cid.to_string());
                            }
                        }
                    }
                    "response.function_call_arguments.done" => {
                        // Arguments are complete — emit FunctionCall immediately
                        let idx = ev
                            .data
                            .get("output_index")
                            .and_then(|i| i.as_i64())
                            .unwrap_or(-1);
                        let name = pending_name.remove(&idx).unwrap_or_default();
                        let call_id = pending_call_id.remove(&idx).unwrap_or_default();
                        let arguments = ev
                            .data
                            .get("arguments")
                            .and_then(|a| a.as_str())
                            .and_then(|s| serde_json::from_str(s).ok())
                            .unwrap_or(serde_json::Value::Object(Default::default()));

                        let fc = crate::types::responses::FunctionCall {
                            call_id,
                            name,
                            arguments,
                        };

                        if fc_tx.send(fc).await.is_err() {
                            break; // receiver dropped
                        }
                    }
                    "response.completed" | "response.failed" => break,
                    _ => {}
                }
            }
        });

        Ok((fc_rx, id_rx))
    }

    /// Retrieve a response by ID.
    ///
    /// `GET /responses/{response_id}`
    pub async fn retrieve(&self, response_id: &str) -> Result<Response, OpenAIError> {
        self.client.get(&format!("/responses/{response_id}")).await
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

        let status = response.status();
        if status.is_success() {
            Ok(())
        } else {
            let status_code = status.as_u16();
            let body = response.text().await.unwrap_or_default();
            if let Ok(error_resp) = serde_json::from_str::<crate::error::ErrorResponse>(&body) {
                Err(OpenAIError::ApiError {
                    status: status_code,
                    message: error_resp.error.message,
                    type_: error_resp.error.type_,
                    code: error_resp.error.code,
                })
            } else {
                Err(OpenAIError::ApiError {
                    status: status_code,
                    message: body,
                    type_: None,
                    code: None,
                })
            }
        }
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
