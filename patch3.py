import sys
with open('src/resources/responses.rs', 'r') as f:
    content = f.read()

target = '''    pub async fn create_stream_raw(
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
            .await?;

        let status = response.status();
        if !status.is_success() {
            let status_code = status.as_u16();
            let body = response.text().await.unwrap_or_default();
            if let Ok(error_resp) = serde_json::from_str::<crate::error::ErrorResponse>(&body) {
                return Err(OpenAIError::ApiError {
                    status: status_code,
                    message: error_resp.error.message,
                    type_: error_resp.error.type_,
                    code: error_resp.error.code,
                });
            }
            return Err(OpenAIError::HttpError {
                status: status_code,
                message: body,
            });
        }
        Ok(crate::streaming::SseStream::new(response))
    }'''

content = content.replace(target, replacement)
with open('src/resources/responses.rs', 'w') as f:
    f.write(content)
