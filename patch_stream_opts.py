import sys

with open('src/resources/chat/mod.rs', 'r') as f:
    content = f.read()

target = '''    pub async fn create_stream(
        &self,
        mut request: ChatCompletionRequest,
    ) -> Result<SseStream<ChatCompletionChunk>, OpenAIError> {
        request.stream = Some(true);'''

replacement = '''    pub async fn create_stream(
        &self,
        mut request: ChatCompletionRequest,
    ) -> Result<SseStream<ChatCompletionChunk>, OpenAIError> {
        request.stream = Some(true);
        // Automatically request usage data for streaming, like AI SDK
        if request.stream_options.is_none() {
            request.stream_options = Some(crate::types::chat::StreamOptions {
                include_usage: Some(true),
            });
        }'''

content = content.replace(target, replacement)
with open('src/resources/chat/mod.rs', 'w') as f:
    f.write(content)

with open('src/types/chat.rs', 'r') as f:
    content2 = f.read()

if "pub struct StreamOptions" not in content2:
    target_req_stream = '''    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,'''
    replacement_req_stream = '''    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// Options for streaming response. Only set this when you set stream: true.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<StreamOptions>,'''
    content2 = content2.replace(target_req_stream, replacement_req_stream)
    
    struct_def = '''
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamOptions {
    /// If set, an additional chunk will be streamed before the data: [DONE] message. 
    /// The usage field on this chunk shows the token usage statistics for the entire request,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_usage: Option<bool>,
}
'''
    content2 = content2 + struct_def
    with open('src/types/chat.rs', 'w') as f:
        f.write(content2)

