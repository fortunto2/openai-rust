import sys
with open('src/resources/responses.rs', 'r') as f:
    content = f.read()

target = '''    pub async fn create_stream_raw(
        &self,
        request: &impl serde::Serialize,
    ) -> Result<crate::streaming::SseStream<serde_json::Value>, OpenAIError> {
        let response = self.client.post_stream_raw("/responses", request).await?;
        Ok(crate::streaming::SseStream::new(response))
    }'''

replacement = '''    pub async fn create_stream_raw(
        &self,
        request: &impl serde::Serialize,
    ) -> Result<crate::streaming::SseStream<serde_json::Value>, OpenAIError> {
        let response = self
            .client
            .request(reqwest::Method::POST, "/responses")
            .header(reqwest::header::ACCEPT, "text/event-stream")
            .header(reqwest::header::CACHE_CONTROL, "no-cache")
            .json(request)
            .send()
            .await
            .map_err(OpenAIError::Reqwest)?;
            
        let response = self.client.handle_response_stream(response).await?;
        Ok(crate::streaming::SseStream::new(response))
    }'''

content = content.replace(target, replacement)
with open('src/resources/responses.rs', 'w') as f:
    f.write(content)
