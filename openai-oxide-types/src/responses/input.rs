// Input items for the Responses API.

use serde::{Deserialize, Serialize};

use super::common::{ImageDetail, Role};
use super::output::{FunctionToolCall, ReasoningItem};

/// Input for the Responses API — text or structured items.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum ResponseInput {
    /// Plain text input.
    Text(String),
    /// Structured message list.
    Messages(Vec<ResponseInputItem>),
    /// Raw items array — for mixed types (messages + function_call_output).
    Items(Vec<serde_json::Value>),
}

impl From<&str> for ResponseInput {
    fn from(s: &str) -> Self {
        ResponseInput::Text(s.to_string())
    }
}

impl From<String> for ResponseInput {
    fn from(s: String) -> Self {
        ResponseInput::Text(s)
    }
}

/// An input message for the Responses API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseInputItem {
    /// Message role.
    pub role: Role,
    /// Message content (text string or structured content array).
    pub content: serde_json::Value,
}

/// A simplified message input with role and content.
///
/// Used for easy construction of user/assistant/system/developer messages.
/// Maps to Python SDK `EasyInputMessage`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EasyInputMessage {
    /// The type of the message input. Always `message`.
    #[serde(rename = "type")]
    pub r#type: MessageType,
    /// The role of the message.
    pub role: Role,
    /// Text or structured content.
    pub content: EasyInputContent,
}

/// Message type marker — always "message".
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub enum MessageType {
    /// Message type.
    #[serde(rename = "message")]
    Message,
}

/// Content for an `EasyInputMessage` — either plain text or a structured content list.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
#[non_exhaustive]
pub enum EasyInputContent {
    /// Plain text content.
    Text(String),
    /// Structured content list (text + images + files).
    ContentList(Vec<InputContent>),
}

/// A single content item within an input message content list.
///
/// Maps to Python SDK `ResponseInputContent` union.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum InputContent {
    /// Text content.
    #[serde(rename = "input_text")]
    InputText(InputTextContent),
    /// Image content.
    #[serde(rename = "input_image")]
    InputImage(InputImageContent),
}

/// Text content within an input message.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InputTextContent {
    /// The text input.
    pub text: String,
}

/// Image content within an input message.
///
/// Maps to Python SDK `ResponseInputImageContent`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputImageContent {
    /// Image detail level.
    pub detail: ImageDetail,
    /// File ID for uploaded images.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
    /// URL or base64 data URL.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
}

/// An input item for the Responses API.
///
/// Union of easy messages and typed items (function calls, reasoning, etc.).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
#[non_exhaustive]
pub enum InputItem {
    /// A simplified message input.
    EasyMessage(EasyInputMessage),
    /// A typed item (function call, function call output, reasoning, etc.).
    Item(Item),
}

/// A typed input item — function call, function call output, or reasoning.
///
/// Maps to the discriminated union in the Python SDK `ResponseInputItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum Item {
    /// A function call from the model.
    #[serde(rename = "function_call")]
    FunctionCall(FunctionToolCall),
    /// Output from a function call (sent back by the client).
    #[serde(rename = "function_call_output")]
    FunctionCallOutput(FunctionCallOutputItemParam),
    /// Reasoning chain-of-thought item.
    #[serde(rename = "reasoning")]
    Reasoning(ReasoningItem),
}

/// Output from a function call, sent back to the model.
///
/// Maps to Python SDK `FunctionCallOutput` input item.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FunctionCallOutputItemParam {
    /// The call ID matching the function call.
    pub call_id: String,
    /// The output content.
    pub output: FunctionCallOutput,
    /// Unique ID (populated when returned via API).
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Item status.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// The output content of a function call.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
#[non_exhaustive]
pub enum FunctionCallOutput {
    /// Plain text output.
    Text(String),
}

/// Additional data to include in the response.
///
/// Maps to Python SDK `ResponseIncludable`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub enum IncludeEnum {
    /// Include file search call results.
    #[serde(rename = "file_search_call.results")]
    FileSearchCallResults,
    /// Include web search call results.
    #[serde(rename = "web_search_call.results")]
    WebSearchCallResults,
    /// Include reasoning encrypted content.
    #[serde(rename = "reasoning.encrypted_content")]
    ReasoningEncryptedContent,
    /// Include message input image URLs.
    #[serde(rename = "message.input_image.image_url")]
    MessageInputImageUrl,
    /// Include computer call output image URLs.
    #[serde(rename = "computer_call_output.output.image_url")]
    ComputerCallOutputImageUrl,
    /// Include code interpreter call outputs.
    #[serde(rename = "code_interpreter_call.outputs")]
    CodeInterpreterCallOutputs,
    /// Include message output text log probabilities.
    #[serde(rename = "message.output_text.logprobs")]
    MessageOutputTextLogprobs,
}
