// SSE stream parser for OpenAI streaming responses

use std::pin::Pin;
use std::task::{Context, Poll};

use futures_core::Stream;

use crate::error::OpenAIError;

/// A stream of parsed SSE events from an OpenAI streaming response.
///
/// See [OpenAI streaming guide](https://platform.openai.com/docs/api-reference/streaming).
///
/// Wraps a byte stream from reqwest and yields deserialized items.
pub struct SseStream<T> {
    #[cfg(not(target_arch = "wasm32"))]
    inner: Pin<Box<dyn Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send>>,
    #[cfg(target_arch = "wasm32")]
    inner: Pin<Box<dyn Stream<Item = Result<bytes::Bytes, reqwest::Error>>>>,
    buffer: String,
    done: bool,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> SseStream<T> {
    pub(crate) fn new(response: reqwest::Response) -> Self {
        Self {
            inner: Box::pin(response.bytes_stream()),
            buffer: String::new(),
            done: false,
            _phantom: std::marker::PhantomData,
        }
    }
}

// SAFETY: SseStream has no self-referential data; inner is heap-boxed.
impl<T> Unpin for SseStream<T> {}

impl<T: serde::de::DeserializeOwned> Stream for SseStream<T> {
    type Item = Result<T, OpenAIError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        loop {
            if this.done {
                return Poll::Ready(None);
            }

            // Check if we already have a complete event in the buffer
            if let Some(item) = try_parse_next::<T>(&mut this.buffer, &mut this.done) {
                return Poll::Ready(Some(item));
            }

            // Poll for more data from the byte stream
            match this.inner.as_mut().poll_next(cx) {
                Poll::Ready(Some(Ok(chunk))) => {
                    this.buffer.push_str(&String::from_utf8_lossy(&chunk));
                    // Safety cap: 4MB max buffer to prevent unbounded growth on malformed streams
                    if this.buffer.len() > 4 * 1024 * 1024 {
                        this.done = true;
                        return Poll::Ready(Some(Err(OpenAIError::StreamError(
                            "SSE buffer exceeded 4MB".into(),
                        ))));
                    }
                    // Loop back to try_parse_next — avoids wake_by_ref() busy-poll.
                    // If no complete event yet, we'll poll inner again which will
                    // either give us more data or return Pending (registering waker).
                    continue;
                }
                Poll::Ready(Some(Err(e))) => {
                    this.done = true;
                    return Poll::Ready(Some(Err(OpenAIError::RequestError(e))));
                }
                Poll::Ready(None) => {
                    this.done = true;
                    return match try_parse_next::<T>(&mut this.buffer, &mut this.done) {
                        Some(item) => Poll::Ready(Some(item)),
                        None => Poll::Ready(None),
                    };
                }
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

/// Try to extract and parse the next SSE event from the buffer.
/// Returns `Some` if an event was found (success or error), `None` if more data is needed.
fn try_parse_next<T: serde::de::DeserializeOwned>(
    buffer: &mut String,
    done: &mut bool,
) -> Option<Result<T, OpenAIError>> {
    loop {
        let newline_pos = buffer.find('\n')?;
        let line = buffer[..newline_pos].trim_end_matches('\r').to_string();
        buffer.drain(..=newline_pos);

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with(':') {
            continue;
        }

        // Parse "data: ..." lines
        if let Some(data) = line
            .strip_prefix("data: ")
            .or_else(|| line.strip_prefix("data:"))
        {
            let data = data.trim();

            if data == "[DONE]" {
                *done = true;
                return None;
            }

            match serde_json::from_str::<T>(data) {
                Ok(value) => return Some(Ok(value)),
                Err(e) => return Some(Err(OpenAIError::JsonError(e))),
            }
        }

        // Skip non-data SSE fields (event:, id:, retry:)
    }
}

/// Parse SSE lines from raw text and yield data payloads.
/// Useful for testing without HTTP. Returns items until `[DONE]` or end of input.
pub fn parse_sse_events<T: serde::de::DeserializeOwned>(raw: &str) -> Vec<Result<T, OpenAIError>> {
    let mut results = Vec::new();
    let mut buffer = raw.to_string();
    if !buffer.ends_with('\n') {
        buffer.push('\n');
    }
    let mut done = false;

    while !done {
        match try_parse_next::<T>(&mut buffer, &mut done) {
            Some(item) => results.push(item),
            None => break,
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::chat::ChatCompletionChunk;

    #[test]
    fn test_parse_sse_content_chunks() {
        let raw = r#"data: {"id":"chatcmpl-1","object":"chat.completion.chunk","created":1,"model":"gpt-4o","choices":[{"index":0,"delta":{"role":"assistant","content":""},"finish_reason":null}]}

data: {"id":"chatcmpl-1","object":"chat.completion.chunk","created":1,"model":"gpt-4o","choices":[{"index":0,"delta":{"content":"Hello"},"finish_reason":null}]}

data: {"id":"chatcmpl-1","object":"chat.completion.chunk","created":1,"model":"gpt-4o","choices":[{"index":0,"delta":{"content":" world"},"finish_reason":null}]}

data: {"id":"chatcmpl-1","object":"chat.completion.chunk","created":1,"model":"gpt-4o","choices":[{"index":0,"delta":{},"finish_reason":"stop"}]}

data: [DONE]

"#;

        let events = parse_sse_events::<ChatCompletionChunk>(raw);
        assert_eq!(events.len(), 4);

        let chunk0 = events[0].as_ref().unwrap();
        assert_eq!(
            chunk0.choices[0].delta.role,
            Some(crate::types::common::Role::Assistant)
        );

        let chunk1 = events[1].as_ref().unwrap();
        assert_eq!(chunk1.choices[0].delta.content.as_deref(), Some("Hello"));

        let chunk2 = events[2].as_ref().unwrap();
        assert_eq!(chunk2.choices[0].delta.content.as_deref(), Some(" world"));

        let chunk3 = events[3].as_ref().unwrap();
        assert_eq!(
            chunk3.choices[0].finish_reason,
            Some(crate::types::common::FinishReason::Stop)
        );
    }

    #[test]
    fn test_parse_sse_with_comments_and_empty_lines() {
        let raw = ": this is a comment
data: {\"id\":\"c1\",\"object\":\"chat.completion.chunk\",\"created\":1,\"model\":\"gpt-4o\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"Hi\"},\"finish_reason\":null}]}

data: [DONE]
";

        let events = parse_sse_events::<ChatCompletionChunk>(raw);
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0].as_ref().unwrap().choices[0]
                .delta
                .content
                .as_deref(),
            Some("Hi")
        );
    }

    #[test]
    fn test_parse_sse_done_stops_parsing() {
        let raw = r#"data: {"id":"c1","object":"chat.completion.chunk","created":1,"model":"gpt-4o","choices":[{"index":0,"delta":{"content":"A"},"finish_reason":null}]}

data: [DONE]

data: {"id":"c2","object":"chat.completion.chunk","created":1,"model":"gpt-4o","choices":[{"index":0,"delta":{"content":"B"},"finish_reason":null}]}
"#;

        let events = parse_sse_events::<ChatCompletionChunk>(raw);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_parse_sse_response_stream_events() {
        use crate::types::responses::ResponseStreamEvent;

        let raw = r#"data: {"type":"response.created","response":{"id":"resp-1","object":"response","created_at":1.0,"model":"gpt-4o","output":[],"status":"in_progress"}}

data: {"type":"response.output_text.delta","delta":"Hello","output_index":0,"content_index":0}

data: {"type":"response.output_text.delta","delta":" world","output_index":0,"content_index":0}

data: {"type":"response.completed","response":{"id":"resp-1","object":"response","created_at":1.0,"model":"gpt-4o","output":[],"status":"completed"}}

data: [DONE]
"#;

        let events = parse_sse_events::<ResponseStreamEvent>(raw);
        assert_eq!(events.len(), 4);
        assert_eq!(events[0].as_ref().unwrap().event_type(), "response.created");
        assert_eq!(
            events[1].as_ref().unwrap().event_type(),
            "response.output_text.delta"
        );
        match events[1].as_ref().unwrap() {
            ResponseStreamEvent::OutputTextDelta { delta, .. } => assert_eq!(delta, "Hello"),
            other => panic!("expected OutputTextDelta, got: {other:?}"),
        }
        match events[2].as_ref().unwrap() {
            ResponseStreamEvent::OutputTextDelta { delta, .. } => assert_eq!(delta, " world"),
            other => panic!("expected OutputTextDelta, got: {other:?}"),
        }
        assert_eq!(
            events[3].as_ref().unwrap().event_type(),
            "response.completed"
        );
    }

    /// Test SSE streaming through actual HTTP (mockito), not just parsing.
    #[tokio::test]
    async fn test_sse_stream_via_http() {
        use futures_util::StreamExt;
        let mut server = mockito::Server::new_async().await;
        let sse_body = "data: {\"id\":\"c1\",\"object\":\"chat.completion.chunk\",\"created\":1,\"model\":\"gpt-4o\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"Hi\"},\"finish_reason\":null}]}\n\ndata: {\"id\":\"c1\",\"object\":\"chat.completion.chunk\",\"created\":1,\"model\":\"gpt-4o\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\" there\"},\"finish_reason\":null}]}\n\ndata: [DONE]\n\n";

        let _mock = server
            .mock("POST", "/chat/completions")
            .with_status(200)
            .with_header("content-type", "text/event-stream")
            .with_body(sse_body)
            .create_async()
            .await;

        let client = crate::OpenAI::with_config(
            crate::config::ClientConfig::new("sk-test").base_url(server.url()),
        );
        let request = crate::types::chat::ChatCompletionRequest::new(
            "gpt-4o",
            vec![crate::types::chat::ChatCompletionMessageParam::User {
                content: crate::types::chat::UserContent::Text("Hi".into()),
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

        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].choices[0].delta.content.as_deref(), Some("Hi"));
        assert_eq!(
            chunks[1].choices[0].delta.content.as_deref(),
            Some(" there")
        );
    }

    /// Test that SSE stream surfaces API errors from the server.
    #[tokio::test]
    async fn test_sse_stream_api_error() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("POST", "/chat/completions")
            .with_status(429)
            .with_body(r#"{"error":{"message":"Rate limit exceeded","type":"rate_limit","param":null,"code":null}}"#)
            .create_async()
            .await;

        let client = crate::OpenAI::with_config(
            crate::config::ClientConfig::new("sk-test")
                .base_url(server.url())
                .max_retries(0),
        );
        let request = crate::types::chat::ChatCompletionRequest::new(
            "gpt-4o",
            vec![crate::types::chat::ChatCompletionMessageParam::User {
                content: crate::types::chat::UserContent::Text("Hi".into()),
                name: None,
            }],
        );
        let err = client
            .chat()
            .completions()
            .create_stream(request)
            .await
            .err()
            .expect("expected error");

        match err {
            OpenAIError::ApiError { status, .. } => assert_eq!(status, 429),
            other => panic!("expected ApiError, got: {other:?}"),
        }
    }

    /// Test SSE with multi-byte UTF-8 that may split across chunks.
    #[test]
    fn test_parse_sse_multibyte_utf8() {
        // Emoji in content
        let raw = "data: {\"id\":\"c1\",\"object\":\"chat.completion.chunk\",\"created\":1,\"model\":\"gpt-4o\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"Hello 🌍\"},\"finish_reason\":null}]}\n\ndata: [DONE]\n";
        let events = parse_sse_events::<ChatCompletionChunk>(raw);
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0].as_ref().unwrap().choices[0]
                .delta
                .content
                .as_deref(),
            Some("Hello 🌍")
        );
    }

    #[test]
    fn test_parse_sse_invalid_json() {
        let raw = "data: {invalid json}\n\ndata: [DONE]\n";
        let events = parse_sse_events::<ChatCompletionChunk>(raw);
        assert_eq!(events.len(), 1);
        assert!(events[0].is_err());
    }

    #[test]
    fn test_parse_sse_tool_call_chunks() {
        let raw = r#"data: {"id":"c1","object":"chat.completion.chunk","created":1,"model":"gpt-4o","choices":[{"index":0,"delta":{"role":"assistant","tool_calls":[{"index":0,"id":"call_1","type":"function","function":{"name":"get_weather","arguments":""}}]},"finish_reason":null}]}

data: {"id":"c1","object":"chat.completion.chunk","created":1,"model":"gpt-4o","choices":[{"index":0,"delta":{"tool_calls":[{"index":0,"function":{"arguments":"{\"loc"}}]},"finish_reason":null}]}

data: {"id":"c1","object":"chat.completion.chunk","created":1,"model":"gpt-4o","choices":[{"index":0,"delta":{"tool_calls":[{"index":0,"function":{"arguments":"ation\": \"Boston\"}"}}]},"finish_reason":null}]}

data: {"id":"c1","object":"chat.completion.chunk","created":1,"model":"gpt-4o","choices":[{"index":0,"delta":{},"finish_reason":"tool_calls"}]}

data: [DONE]
"#;

        let events = parse_sse_events::<ChatCompletionChunk>(raw);
        assert_eq!(events.len(), 4);

        let tc = events[0].as_ref().unwrap().choices[0]
            .delta
            .tool_calls
            .as_ref()
            .unwrap();
        assert_eq!(tc[0].id.as_deref(), Some("call_1"));
        assert_eq!(
            tc[0].function.as_ref().unwrap().name.as_deref(),
            Some("get_weather")
        );

        assert_eq!(
            events[3].as_ref().unwrap().choices[0].finish_reason,
            Some(crate::types::common::FinishReason::ToolCalls)
        );
    }
}
