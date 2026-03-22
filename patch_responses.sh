#!/bin/bash
# Patch src/resources/responses.rs to add create_stream_raw
cat << 'PATCH_EOF' >> src/resources/responses.rs

    /// Create a streaming response using raw generic JSON types.
    pub async fn create_stream_raw(
        &self,
        request: &impl serde::Serialize,
    ) -> Result<SseStream<serde_json::Value>, OpenAIError> {
        let response = self.client.post_stream_raw("/responses", request).await?;
        Ok(SseStream::new(response))
    }
PATCH_EOF
