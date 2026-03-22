import sys

with open('src/resources/responses.rs', 'r') as f:
    content = f.read()

target = '''    pub async fn create_stream(
        &self,
        mut request: ResponseCreateRequest,
    ) -> Result<SseStream<ResponseStreamEvent>, OpenAIError> {
        request.stream = Some(true);'''

replacement = '''    pub async fn create_stream(
        &self,
        mut request: ResponseCreateRequest,
    ) -> Result<SseStream<ResponseStreamEvent>, OpenAIError> {
        request.stream = Some(true);
        // We always receive usage from Responses API automatically or stream_options doesn't apply the same way,
        // but let's make sure reasoning fields are aligned too if needed.
        // Actually, Response API does not use StreamOptions, it sends usage automatically.'''

content = content.replace(target, replacement)
with open('src/resources/responses.rs', 'w') as f:
    f.write(content)
