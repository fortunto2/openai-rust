// High-level streaming helpers — typed events + delta accumulation.
//
// Wraps the raw `SseStream<ChatCompletionChunk>` with automatic state
// tracking, text accumulation, and tool call assembly.

use std::pin::Pin;
use std::task::{Context, Poll};

use futures_core::Stream;
use futures_util::StreamExt;

use crate::error::OpenAIError;
use crate::streaming::SseStream;
use crate::types::chat::{
    ChatCompletionChunk, ChatCompletionMessage, ChatCompletionResponse, ChunkChoice,
};
use crate::types::common::{FinishReason, Role, Usage};

// ── Event types ──

/// A high-level streaming event with accumulated state.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ChatStreamEvent {
    /// Raw chunk received (always emitted).
    Chunk(Box<ChatCompletionChunk>),

    /// Text content delta.
    ContentDelta {
        /// The new text fragment.
        delta: String,
        /// Full accumulated text so far.
        snapshot: String,
    },

    /// Text content is complete.
    ContentDone {
        /// The full accumulated text.
        content: String,
    },

    /// Refusal delta.
    RefusalDelta { delta: String, snapshot: String },

    /// Refusal is complete.
    RefusalDone { refusal: String },

    /// Tool call arguments are still arriving.
    ToolCallDelta {
        /// Tool call index.
        index: i32,
        /// Function name (set on first delta).
        name: String,
        /// New arguments fragment.
        arguments_delta: String,
        /// Full accumulated arguments so far.
        arguments_snapshot: String,
    },

    /// Tool call is complete — arguments are ready to use.
    ToolCallDone {
        /// Tool call index.
        index: i32,
        /// The tool call ID.
        call_id: String,
        /// Function name.
        name: String,
        /// Full JSON arguments string.
        arguments: String,
    },

    /// Stream finished.
    Done { finish_reason: Option<FinishReason> },
}

// ── Accumulator state ──

/// Tracks state for a single tool call being assembled from deltas.
#[derive(Debug, Clone, Default)]
struct ToolCallState {
    id: String,
    name: String,
    arguments: String,
}

/// Accumulates streaming chunks into a coherent state.
#[derive(Debug)]
struct StreamState {
    id: String,
    model: String,
    created: i64,
    content: String,
    refusal: String,
    tool_calls: Vec<ToolCallState>,
    finish_reason: Option<FinishReason>,
    usage: Option<Usage>,
    system_fingerprint: Option<String>,
    service_tier: Option<crate::types::common::ServiceTier>,
    content_done: bool,
    refusal_done: bool,
    tools_done: Vec<bool>,
}

impl StreamState {
    fn new() -> Self {
        Self {
            id: String::new(),
            model: String::new(),
            created: 0,
            content: String::new(),
            refusal: String::new(),
            tool_calls: Vec::new(),
            finish_reason: None,
            usage: None,
            system_fingerprint: None,
            service_tier: None,
            content_done: false,
            refusal_done: false,
            tools_done: Vec::new(),
        }
    }

    /// Process a chunk and return high-level events.
    fn handle_chunk(&mut self, chunk: &ChatCompletionChunk) -> Vec<ChatStreamEvent> {
        let mut events = Vec::new();

        // Update metadata from first chunk
        if self.id.is_empty() {
            self.id.clone_from(&chunk.id);
            self.model.clone_from(&chunk.model);
            self.created = chunk.created;
        }
        if chunk.system_fingerprint.is_some() {
            self.system_fingerprint
                .clone_from(&chunk.system_fingerprint);
        }
        if chunk.service_tier.is_some() {
            self.service_tier = chunk.service_tier.clone();
        }
        if chunk.usage.is_some() {
            self.usage.clone_from(&chunk.usage);
        }

        for choice in &chunk.choices {
            self.handle_choice(choice, &mut events);
        }

        events
    }

    fn handle_choice(&mut self, choice: &ChunkChoice, events: &mut Vec<ChatStreamEvent>) {
        let delta = &choice.delta;

        // Content delta
        if let Some(ref text) = delta.content
            && !text.is_empty()
        {
            self.content.push_str(text);
            events.push(ChatStreamEvent::ContentDelta {
                delta: text.clone(),
                snapshot: self.content.clone(),
            });
        }

        // Refusal delta
        if let Some(ref refusal) = delta.refusal
            && !refusal.is_empty()
        {
            self.refusal.push_str(refusal);
            events.push(ChatStreamEvent::RefusalDelta {
                delta: refusal.clone(),
                snapshot: self.refusal.clone(),
            });
        }

        // Tool call deltas
        if let Some(ref tool_calls) = delta.tool_calls {
            for tc in tool_calls {
                let idx = tc.index as usize;

                // Ensure we have state for this tool call
                while self.tool_calls.len() <= idx {
                    self.tool_calls.push(ToolCallState::default());
                    self.tools_done.push(false);
                }

                let state = &mut self.tool_calls[idx];

                // First delta for this tool call — capture id and name
                if let Some(ref id) = tc.id {
                    state.id = id.clone();
                }
                if let Some(ref func) = tc.function {
                    if let Some(ref name) = func.name {
                        state.name = name.clone();
                    }
                    if let Some(ref args) = func.arguments {
                        state.arguments.push_str(args);
                        events.push(ChatStreamEvent::ToolCallDelta {
                            index: tc.index,
                            name: state.name.clone(),
                            arguments_delta: args.clone(),
                            arguments_snapshot: state.arguments.clone(),
                        });
                    }
                }
            }
        }

        // Finish reason — emit done events
        if let Some(ref fr) = choice.finish_reason {
            self.finish_reason = Some(fr.clone());

            // Content done
            if !self.content.is_empty() && !self.content_done {
                self.content_done = true;
                events.push(ChatStreamEvent::ContentDone {
                    content: self.content.clone(),
                });
            }

            // Refusal done
            if !self.refusal.is_empty() && !self.refusal_done {
                self.refusal_done = true;
                events.push(ChatStreamEvent::RefusalDone {
                    refusal: self.refusal.clone(),
                });
            }

            // Tool calls done
            for (i, tc) in self.tool_calls.iter().enumerate() {
                if !self.tools_done[i] {
                    self.tools_done[i] = true;
                    events.push(ChatStreamEvent::ToolCallDone {
                        index: i as i32,
                        call_id: tc.id.clone(),
                        name: tc.name.clone(),
                        arguments: tc.arguments.clone(),
                    });
                }
            }

            events.push(ChatStreamEvent::Done {
                finish_reason: Some(fr.clone()),
            });
        }
    }

    /// Build a full ChatCompletionResponse from accumulated state.
    fn into_completion(self) -> ChatCompletionResponse {
        let tool_calls = if self.tool_calls.is_empty() {
            None
        } else {
            Some(
                self.tool_calls
                    .into_iter()
                    .map(|tc| crate::types::chat::ToolCall {
                        id: tc.id,
                        type_: "function".into(),
                        function: crate::types::chat::FunctionCall {
                            name: tc.name,
                            arguments: tc.arguments,
                        },
                    })
                    .collect(),
            )
        };

        ChatCompletionResponse {
            id: self.id,
            choices: vec![crate::types::chat::ChatCompletionChoice {
                index: 0,
                finish_reason: self.finish_reason.unwrap_or(FinishReason::Stop),
                message: ChatCompletionMessage {
                    role: Role::Assistant,
                    content: if self.content.is_empty() {
                        None
                    } else {
                        Some(self.content)
                    },
                    refusal: if self.refusal.is_empty() {
                        None
                    } else {
                        Some(self.refusal)
                    },
                    tool_calls,
                    annotations: None,
                },
                logprobs: None,
            }],
            created: self.created,
            model: self.model,
            object: "chat.completion".into(),
            service_tier: self.service_tier,
            system_fingerprint: self.system_fingerprint,
            usage: self.usage,
        }
    }
}

// ── ChatCompletionStream ──

/// High-level streaming wrapper with typed events and accumulation.
///
/// Wraps the raw SSE stream and yields [`ChatStreamEvent`]s instead of
/// raw chunks. Automatically accumulates text, tool calls, and usage.
///
/// ```ignore
/// use openai_oxide::stream_helpers::ChatStreamEvent;
///
/// let mut stream = client.chat().completions()
///     .create_stream_helper(request).await?;
///
/// while let Some(event) = stream.next().await {
///     match event? {
///         ChatStreamEvent::ContentDelta { delta, .. } => print!("{delta}"),
///         ChatStreamEvent::ToolCallDone { name, arguments, .. } => {
///             execute_tool(&name, &arguments).await;
///         }
///         ChatStreamEvent::Done { .. } => break,
///         _ => {}
///     }
/// }
///
/// // Or just get the final result:
/// let completion = stream.get_final_completion().await?;
/// ```
pub struct ChatCompletionStream {
    inner: SseStream<ChatCompletionChunk>,
    state: StreamState,
    event_buffer: Vec<ChatStreamEvent>,
    done: bool,
}

impl ChatCompletionStream {
    pub(crate) fn new(inner: SseStream<ChatCompletionChunk>) -> Self {
        Self {
            inner,
            state: StreamState::new(),
            event_buffer: Vec::new(),
            done: false,
        }
    }

    /// Consume the stream and return the accumulated completion.
    ///
    /// Equivalent to iterating through all events and building the response.
    pub async fn get_final_completion(mut self) -> Result<ChatCompletionResponse, OpenAIError> {
        while let Some(result) = self.inner.next().await {
            let chunk = result?;
            self.state.handle_chunk(&chunk);
        }
        Ok(self.state.into_completion())
    }

    /// Get the current accumulated text content.
    pub fn current_content(&self) -> &str {
        &self.state.content
    }
}

impl Stream for ChatCompletionStream {
    type Item = Result<ChatStreamEvent, OpenAIError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        // Drain buffered events first
        if !this.event_buffer.is_empty() {
            return Poll::Ready(Some(Ok(this.event_buffer.remove(0))));
        }

        if this.done {
            return Poll::Ready(None);
        }

        match this.inner.poll_next_unpin(cx) {
            Poll::Ready(Some(Ok(chunk))) => {
                let mut events = this.state.handle_chunk(&chunk);

                // Always emit the raw chunk first
                let first = ChatStreamEvent::Chunk(Box::new(chunk));

                if events.is_empty() {
                    Poll::Ready(Some(Ok(first)))
                } else {
                    // Buffer remaining events, return first
                    this.event_buffer.append(&mut events);
                    Poll::Ready(Some(Ok(first)))
                }
            }
            Poll::Ready(Some(Err(e))) => {
                this.done = true;
                Poll::Ready(Some(Err(e)))
            }
            Poll::Ready(None) => {
                this.done = true;
                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

// SAFETY: ChatCompletionStream's inner is heap-boxed, no self-referential data.
impl Unpin for ChatCompletionStream {}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_chunk(
        content: Option<&str>,
        finish_reason: Option<FinishReason>,
    ) -> ChatCompletionChunk {
        ChatCompletionChunk {
            id: "c1".into(),
            choices: vec![ChunkChoice {
                delta: crate::types::chat::ChoiceDelta {
                    content: content.map(String::from),
                    role: None,
                    refusal: None,
                    tool_calls: None,
                },
                finish_reason,
                index: 0,
                logprobs: None,
            }],
            created: 1,
            model: "gpt-4o".into(),
            object: "chat.completion.chunk".into(),
            service_tier: None,
            system_fingerprint: None,
            usage: None,
        }
    }

    #[test]
    fn test_accumulate_content() {
        let mut state = StreamState::new();

        let events = state.handle_chunk(&make_chunk(Some("Hello"), None));
        assert_eq!(events.len(), 1);
        match &events[0] {
            ChatStreamEvent::ContentDelta { delta, snapshot } => {
                assert_eq!(delta, "Hello");
                assert_eq!(snapshot, "Hello");
            }
            other => panic!("expected ContentDelta, got: {other:?}"),
        }

        let events = state.handle_chunk(&make_chunk(Some(" world"), None));
        match &events[0] {
            ChatStreamEvent::ContentDelta { delta, snapshot } => {
                assert_eq!(delta, " world");
                assert_eq!(snapshot, "Hello world");
            }
            other => panic!("expected ContentDelta, got: {other:?}"),
        }
    }

    #[test]
    fn test_content_done_on_finish() {
        let mut state = StreamState::new();
        state.handle_chunk(&make_chunk(Some("Hi"), None));

        let events = state.handle_chunk(&make_chunk(None, Some(FinishReason::Stop)));
        assert!(
            events
                .iter()
                .any(|e| matches!(e, ChatStreamEvent::ContentDone { content } if content == "Hi"))
        );
        assert!(events.iter().any(|e| matches!(
            e,
            ChatStreamEvent::Done {
                finish_reason: Some(FinishReason::Stop)
            }
        )));
    }

    #[test]
    fn test_tool_call_accumulation() {
        let mut state = StreamState::new();

        // First chunk: tool call name
        let chunk1 = ChatCompletionChunk {
            id: "c1".into(),
            choices: vec![ChunkChoice {
                delta: crate::types::chat::ChoiceDelta {
                    content: None,
                    role: Some(Role::Assistant),
                    refusal: None,
                    tool_calls: Some(vec![crate::types::chat::DeltaToolCall {
                        index: 0,
                        id: Some("call_1".into()),
                        function: Some(crate::types::chat::DeltaFunctionCall {
                            name: Some("get_weather".into()),
                            arguments: Some("{\"loc".into()),
                        }),
                        type_: Some("function".into()),
                    }]),
                },
                finish_reason: None,
                index: 0,
                logprobs: None,
            }],
            created: 1,
            model: "gpt-4o".into(),
            object: "chat.completion.chunk".into(),
            service_tier: None,
            system_fingerprint: None,
            usage: None,
        };

        let events = state.handle_chunk(&chunk1);
        assert!(events.iter().any(
            |e| matches!(e, ChatStreamEvent::ToolCallDelta { name, .. } if name == "get_weather")
        ));

        // Second chunk: more arguments
        let chunk2 = ChatCompletionChunk {
            id: "c1".into(),
            choices: vec![ChunkChoice {
                delta: crate::types::chat::ChoiceDelta {
                    content: None,
                    role: None,
                    refusal: None,
                    tool_calls: Some(vec![crate::types::chat::DeltaToolCall {
                        index: 0,
                        id: None,
                        function: Some(crate::types::chat::DeltaFunctionCall {
                            name: None,
                            arguments: Some("ation\": \"SF\"}".into()),
                        }),
                        type_: None,
                    }]),
                },
                finish_reason: None,
                index: 0,
                logprobs: None,
            }],
            created: 1,
            model: "gpt-4o".into(),
            object: "chat.completion.chunk".into(),
            service_tier: None,
            system_fingerprint: None,
            usage: None,
        };

        state.handle_chunk(&chunk2);

        // Finish
        let events = state.handle_chunk(&make_chunk(None, Some(FinishReason::ToolCalls)));
        assert!(events.iter().any(|e| matches!(
            e,
            ChatStreamEvent::ToolCallDone { name, arguments, call_id, .. }
            if name == "get_weather" && arguments == "{\"location\": \"SF\"}" && call_id == "call_1"
        )));
    }

    #[test]
    fn test_into_completion() {
        let mut state = StreamState::new();
        state.handle_chunk(&make_chunk(Some("Hello"), None));
        state.handle_chunk(&make_chunk(Some(" world"), None));
        state.handle_chunk(&make_chunk(None, Some(FinishReason::Stop)));

        let completion = state.into_completion();
        assert_eq!(
            completion.choices[0].message.content.as_deref(),
            Some("Hello world")
        );
        assert_eq!(completion.choices[0].finish_reason, FinishReason::Stop);
    }
}
