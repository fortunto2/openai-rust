// SSE stream parser for OpenAI streaming responses

use std::pin::Pin;
use std::task::{Context, Poll};

use futures_core::Stream;

use crate::error::OpenAIError;

/// A stream of parsed SSE events from an OpenAI streaming response.
///
/// Wraps a byte stream from reqwest and yields deserialized items.
pub struct SseStream<T> {
    inner: Pin<Box<dyn Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send>>,
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
                match try_parse_next::<T>(&mut this.buffer, &mut this.done) {
                    Some(item) => Poll::Ready(Some(item)),
                    None => {
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    }
                }
            }
            Poll::Ready(Some(Err(e))) => {
                this.done = true;
                Poll::Ready(Some(Err(OpenAIError::RequestError(e))))
            }
            Poll::Ready(None) => {
                this.done = true;
                match try_parse_next::<T>(&mut this.buffer, &mut this.done) {
                    Some(item) => Poll::Ready(Some(item)),
                    None => Poll::Ready(None),
                }
            }
            Poll::Pending => Poll::Pending,
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

        let raw = r#"data: {"type":"response.created","response":{"id":"resp-1","object":"response","status":"in_progress"}}

data: {"type":"response.output_text.delta","delta":"Hello","output_index":0,"content_index":0}

data: {"type":"response.output_text.delta","delta":" world","output_index":0,"content_index":0}

data: {"type":"response.completed","response":{"id":"resp-1","status":"completed"}}

data: [DONE]
"#;

        let events = parse_sse_events::<ResponseStreamEvent>(raw);
        assert_eq!(events.len(), 4);
        assert_eq!(events[0].as_ref().unwrap().type_, "response.created");
        assert_eq!(
            events[1].as_ref().unwrap().type_,
            "response.output_text.delta"
        );
        assert_eq!(events[1].as_ref().unwrap().data["delta"], "Hello");
        assert_eq!(events[2].as_ref().unwrap().data["delta"], " world");
        assert_eq!(events[3].as_ref().unwrap().type_, "response.completed");
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
