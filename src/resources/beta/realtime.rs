// Realtime resource — client.beta().realtime().sessions().create()

use super::BETA_HEADER;
use crate::client::OpenAI;
use crate::error::OpenAIError;
use crate::types::realtime::{SessionCreateRequest, SessionCreateResponse};

/// Access Realtime API endpoints (beta).
pub struct Realtime<'a> {
    client: &'a OpenAI,
}

impl<'a> Realtime<'a> {
    pub(crate) fn new(client: &'a OpenAI) -> Self {
        Self { client }
    }

    /// Access the sessions sub-resource.
    pub fn sessions(&self) -> Sessions<'_> {
        Sessions {
            client: self.client,
        }
    }
}

/// Realtime sessions endpoint.
pub struct Sessions<'a> {
    client: &'a OpenAI,
}

impl<'a> Sessions<'a> {
    /// Create a new realtime session with an ephemeral token.
    ///
    /// `POST /realtime/sessions`
    pub async fn create(
        &self,
        request: SessionCreateRequest,
    ) -> Result<SessionCreateResponse, OpenAIError> {
        let response = self
            .client
            .request(reqwest::Method::POST, "/realtime/sessions")
            .header(BETA_HEADER.0, BETA_HEADER.1)
            .json(&request)
            .send()
            .await?;
        OpenAI::handle_response(response).await
    }
}

#[cfg(test)]
mod tests {
    use crate::OpenAI;
    use crate::config::ClientConfig;
    use crate::types::realtime::SessionCreateRequest;

    #[tokio::test]
    async fn test_realtime_session_create() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/realtime/sessions")
            .match_header("authorization", "Bearer sk-test")
            .match_header("openai-beta", "assistants=v2")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "client_secret": {
                        "value": "ek-abc123",
                        "expires_at": 1700000000
                    },
                    "model": "gpt-4o-realtime-preview",
                    "voice": "alloy",
                    "modalities": ["text", "audio"],
                    "temperature": 0.8,
                    "input_audio_format": "pcm16",
                    "output_audio_format": "pcm16"
                }"#,
            )
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        let request = SessionCreateRequest::new()
            .model("gpt-4o-realtime-preview")
            .voice("alloy")
            .modalities(vec!["text".into(), "audio".into()]);

        let response = client
            .beta()
            .realtime()
            .sessions()
            .create(request)
            .await
            .unwrap();
        assert_eq!(response.client_secret.value, "ek-abc123");
        assert_eq!(response.model, Some("gpt-4o-realtime-preview".into()));
        assert_eq!(response.voice, Some("alloy".into()));
        mock.assert_async().await;
    }
}
