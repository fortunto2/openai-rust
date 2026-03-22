import sys
with open('src/resources/responses.rs', 'r') as f:
    content = f.read()

target = 'self.client.post_json("/responses", request).await\n    }'
replacement = target + '''

    pub async fn create_stream_raw(
        &self,
        request: &impl serde::Serialize,
    ) -> Result<crate::streaming::SseStream<serde_json::Value>, OpenAIError> {
        let response = self.client.post_stream_raw("/responses", request).await?;
        Ok(crate::streaming::SseStream::new(response))
    }'''

content = content.replace(target, replacement)
with open('src/resources/responses.rs', 'w') as f:
    f.write(content)
