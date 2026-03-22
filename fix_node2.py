import sys

with open('src/resources/chat/mod.rs', 'r') as f:
    content = f.read()

# Add create_stream_raw to chat mod
new_method = '''
    pub async fn create_stream_raw(
        &self,
        request: &impl serde::Serialize,
    ) -> Result<crate::streaming::SseStream<serde_json::Value>, OpenAIError> {
        let response = self
            .client
            .request(reqwest::Method::POST, "/chat/completions")
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
            return Err(OpenAIError::ApiError {
                status: status_code,
                message: body,
                type_: None,
                code: None,
            });
        }

        Ok(crate::streaming::SseStream::new(response))
    }
'''

if "create_stream_raw" not in content:
    content = content.replace("impl<'a> Completions<'a> {", "impl<'a> Completions<'a> {" + new_method)

with open('src/resources/chat/mod.rs', 'w') as f:
    f.write(content)

