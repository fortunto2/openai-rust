// Streaming event types for the Responses API.

use serde::{Deserialize, Serialize};

use super::output::{OutputItem, ResponseOutputItem};
use super::response::Response;

/// Emitted when there is a text delta.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTextDeltaEvent {
    /// Sequence number.
    #[serde(default)]
    pub sequence_number: i64,
    /// The ID of the output item.
    #[serde(default)]
    pub item_id: String,
    /// Index of the output item.
    #[serde(default)]
    pub output_index: i64,
    /// Index of the content part.
    #[serde(default)]
    pub content_index: i64,
    /// The text delta.
    pub delta: String,
    /// Log probabilities (if requested).
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<serde_json::Value>,
}

/// Emitted when there is a reasoning text delta.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseReasoningTextDeltaEvent {
    /// Sequence number.
    #[serde(default)]
    pub sequence_number: i64,
    /// The ID of the item.
    #[serde(default)]
    pub item_id: String,
    /// Index of the output item.
    #[serde(default)]
    pub output_index: i64,
    /// Index of the content part.
    #[serde(default)]
    pub content_index: i64,
    /// The reasoning text delta.
    pub delta: String,
}

/// Emitted when there is a reasoning summary text delta.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseReasoningSummaryTextDeltaEvent {
    /// Sequence number.
    #[serde(default)]
    pub sequence_number: i64,
    /// The ID of the item.
    #[serde(default)]
    pub item_id: String,
    /// Index of the output item.
    #[serde(default)]
    pub output_index: i64,
    /// Index of the summary part.
    #[serde(default)]
    pub summary_index: i64,
    /// The summary text delta.
    pub delta: String,
}

/// Emitted when a new output item is added.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseOutputItemAddedEvent {
    /// Sequence number.
    #[serde(default)]
    pub sequence_number: i64,
    /// Index of the output item.
    #[serde(default)]
    pub output_index: i64,
    /// The output item.
    pub item: OutputItem,
}

/// Emitted when there is a function call arguments delta.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseFunctionCallArgumentsDeltaEvent {
    /// Sequence number.
    #[serde(default)]
    pub sequence_number: i64,
    /// The ID of the item.
    #[serde(default)]
    pub item_id: String,
    /// Index of the output item.
    #[serde(default)]
    pub output_index: u32,
    /// The arguments delta.
    pub delta: String,
}

/// Emitted when function call arguments are finalized.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseFunctionCallArgumentsDoneEvent {
    /// Sequence number.
    #[serde(default)]
    pub sequence_number: i64,
    /// The ID of the item.
    #[serde(default)]
    pub item_id: String,
    /// Index of the output item.
    #[serde(default)]
    pub output_index: u32,
    /// The complete arguments JSON.
    pub arguments: String,
    /// The function name.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Emitted when the response is complete.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseCompletedEvent {
    /// Sequence number.
    #[serde(default)]
    pub sequence_number: i64,
    /// The completed response.
    pub response: Response,
}

/// Emitted when the response fails.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseFailedEvent {
    /// Sequence number.
    #[serde(default)]
    pub sequence_number: i64,
    /// The failed response.
    pub response: Response,
}

/// Emitted when the response is incomplete.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseIncompleteEvent {
    /// Sequence number.
    #[serde(default)]
    pub sequence_number: i64,
    /// The incomplete response.
    pub response: Response,
}

/// Emitted when an error occurs during streaming.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseErrorEvent {
    /// Sequence number.
    #[serde(default)]
    pub sequence_number: i64,
    /// Error message.
    pub message: String,
    /// Error code.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// Error parameter.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param: Option<String>,
}

/// A streaming event from the Responses API.
///
/// Uses `#[serde(tag = "type")]` for typed deserialization. Unknown event types
/// fall through to the `Other` variant to ensure forward compatibility.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum ResponseStreamEvent {
    // -- Lifecycle events --
    /// Response created.
    #[serde(rename = "response.created")]
    ResponseCreated {
        /// The created response.
        response: Response,
    },
    /// Response in progress.
    #[serde(rename = "response.in_progress")]
    ResponseInProgress {
        /// The in-progress response.
        response: Response,
    },
    /// Response completed.
    #[serde(rename = "response.completed")]
    ResponseCompleted(ResponseCompletedEvent),
    /// Response failed.
    #[serde(rename = "response.failed")]
    ResponseFailed(ResponseFailedEvent),
    /// Response incomplete.
    #[serde(rename = "response.incomplete")]
    ResponseIncomplete(ResponseIncompleteEvent),

    // -- Output item events --
    /// New output item added.
    #[serde(rename = "response.output_item.added")]
    ResponseOutputItemAdded(ResponseOutputItemAddedEvent),
    /// Output item done.
    #[serde(rename = "response.output_item.done")]
    OutputItemDone {
        /// Index of the output item.
        output_index: i64,
        /// The completed output item.
        item: ResponseOutputItem,
    },

    // -- Content part events --
    /// Content part added.
    #[serde(rename = "response.content_part.added")]
    ContentPartAdded {
        /// Index of the output item.
        output_index: i64,
        /// Index of the content part.
        content_index: i64,
        /// The content part.
        part: serde_json::Value,
    },
    /// Content part done.
    #[serde(rename = "response.content_part.done")]
    ContentPartDone {
        /// Index of the output item.
        output_index: i64,
        /// Index of the content part.
        content_index: i64,
        /// The content part.
        part: serde_json::Value,
    },

    // -- Text delta events --
    /// Text output delta.
    #[serde(rename = "response.output_text.delta")]
    ResponseOutputTextDelta(ResponseTextDeltaEvent),
    /// Text output done.
    #[serde(rename = "response.output_text.done")]
    OutputTextDone {
        /// Index of the output item.
        output_index: i64,
        /// Index of the content part.
        content_index: i64,
        /// The complete text.
        text: String,
    },

    // -- Function call events --
    /// Function call arguments delta.
    #[serde(rename = "response.function_call_arguments.delta")]
    ResponseFunctionCallArgumentsDelta(ResponseFunctionCallArgumentsDeltaEvent),
    /// Function call arguments done.
    #[serde(rename = "response.function_call_arguments.done")]
    ResponseFunctionCallArgumentsDone(ResponseFunctionCallArgumentsDoneEvent),

    // -- Reasoning events --
    /// Reasoning text delta.
    #[serde(rename = "response.reasoning_text.delta")]
    ResponseReasoningTextDelta(ResponseReasoningTextDeltaEvent),
    /// Reasoning summary text delta.
    #[serde(rename = "response.reasoning_summary_text.delta")]
    ResponseReasoningSummaryTextDelta(ResponseReasoningSummaryTextDeltaEvent),
    /// Reasoning summary text done.
    #[serde(rename = "response.reasoning_summary_text.done")]
    ReasoningSummaryTextDone {
        /// Index of the output item.
        output_index: i64,
        /// Index of the summary part.
        #[serde(default)]
        summary_index: Option<i64>,
        /// The complete summary text.
        text: String,
    },

    // -- Error event --
    /// Error event.
    #[serde(rename = "error")]
    ResponseError(ResponseErrorEvent),

    // -- Catch-all for unknown/new event types --
    /// Unknown event type. Contains the raw JSON data for forward compatibility.
    #[serde(untagged)]
    Other(serde_json::Value),
}

impl ResponseStreamEvent {
    /// Get the event type string.
    pub fn event_type(&self) -> &str {
        match self {
            Self::ResponseCreated { .. } => "response.created",
            Self::ResponseInProgress { .. } => "response.in_progress",
            Self::ResponseCompleted { .. } => "response.completed",
            Self::ResponseFailed { .. } => "response.failed",
            Self::ResponseIncomplete { .. } => "response.incomplete",
            Self::ResponseOutputItemAdded { .. } => "response.output_item.added",
            Self::OutputItemDone { .. } => "response.output_item.done",
            Self::ContentPartAdded { .. } => "response.content_part.added",
            Self::ContentPartDone { .. } => "response.content_part.done",
            Self::ResponseOutputTextDelta { .. } => "response.output_text.delta",
            Self::OutputTextDone { .. } => "response.output_text.done",
            Self::ResponseFunctionCallArgumentsDelta { .. } => {
                "response.function_call_arguments.delta"
            }
            Self::ResponseFunctionCallArgumentsDone { .. } => {
                "response.function_call_arguments.done"
            }
            Self::ResponseReasoningTextDelta { .. } => "response.reasoning_text.delta",
            Self::ResponseReasoningSummaryTextDelta { .. } => {
                "response.reasoning_summary_text.delta"
            }
            Self::ReasoningSummaryTextDone { .. } => "response.reasoning_summary_text.done",
            Self::ResponseError { .. } => "error",
            Self::Other(v) => v.get("type").and_then(|t| t.as_str()).unwrap_or("unknown"),
        }
    }
}
