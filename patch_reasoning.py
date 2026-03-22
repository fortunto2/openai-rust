import sys

with open('src/resources/chat/mod.rs', 'r') as f:
    content = f.read()

target = '''    pub async fn create(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, OpenAIError> {
        self.client.post("/chat/completions", &request).await
    }'''

replacement = '''    pub async fn create(
        &self,
        mut request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, OpenAIError> {
        Self::prepare_reasoning_request(&mut request);
        self.client.post("/chat/completions", &request).await
    }'''

content = content.replace(target, replacement)

target2 = '''    pub async fn create_stream(
        &self,
        mut request: ChatCompletionRequest,
    ) -> Result<SseStream<ChatCompletionChunk>, OpenAIError> {'''

replacement2 = '''    pub async fn create_stream(
        &self,
        mut request: ChatCompletionRequest,
    ) -> Result<SseStream<ChatCompletionChunk>, OpenAIError> {
        Self::prepare_reasoning_request(&mut request);'''

content = content.replace(target2, replacement2)

helper = '''

    /// Automatically aligns parameters for O1/O3 reasoning models to prevent API errors.
    fn prepare_reasoning_request(request: &mut ChatCompletionRequest) {
        if request.model.starts_with("o1") || request.model.starts_with("o3") {
            // Reasoning models crash if temperature or other generation parameters are passed
            if request.temperature.is_some() {
                tracing::warn!("temperature is not supported for reasoning models. Dropping parameter.");
                request.temperature = None;
            }
            if request.top_p.is_some() {
                tracing::warn!("top_p is not supported for reasoning models. Dropping parameter.");
                request.top_p = None;
            }
            if request.presence_penalty.is_some() {
                tracing::warn!("presence_penalty is not supported for reasoning models. Dropping parameter.");
                request.presence_penalty = None;
            }
            if request.frequency_penalty.is_some() {
                tracing::warn!("frequency_penalty is not supported for reasoning models. Dropping parameter.");
                request.frequency_penalty = None;
            }
            
            // Map max_tokens -> max_completion_tokens
            if request.max_tokens.is_some() && request.max_completion_tokens.is_none() {
                tracing::debug!("Mapping max_tokens to max_completion_tokens for reasoning model");
                request.max_completion_tokens = request.max_tokens;
                request.max_tokens = None;
            }
        }
    }'''

if "prepare_reasoning_request" not in content:
    content = content.replace("impl<'a> Completions<'a> {", "impl<'a> Completions<'a> {" + helper)

with open('src/resources/chat/mod.rs', 'w') as f:
    f.write(content)
