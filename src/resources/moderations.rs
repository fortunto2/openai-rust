// Moderations resource — client.moderations().create()

use crate::client::OpenAI;
use crate::error::OpenAIError;
use crate::types::moderation::{ModerationCreateResponse, ModerationRequest};

/// Access moderation endpoints.
pub struct Moderations<'a> {
    client: &'a OpenAI,
}

impl<'a> Moderations<'a> {
    pub(crate) fn new(client: &'a OpenAI) -> Self {
        Self { client }
    }

    /// Classify text for potentially harmful content.
    ///
    /// `POST /moderations`
    pub async fn create(
        &self,
        request: ModerationRequest,
    ) -> Result<ModerationCreateResponse, OpenAIError> {
        self.client.post("/moderations", &request).await
    }
}

#[cfg(test)]
mod tests {
    use crate::config::ClientConfig;
    use crate::types::moderation::ModerationRequest;
    use crate::OpenAI;

    #[tokio::test]
    async fn test_moderations_create() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/moderations")
            .match_header("authorization", "Bearer sk-test")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "id": "modr-abc123",
                    "model": "text-moderation-007",
                    "results": [{
                        "flagged": false,
                        "categories": {
                            "harassment": false,
                            "harassment/threatening": false,
                            "hate": false,
                            "hate/threatening": false,
                            "self-harm": false,
                            "self-harm/instructions": false,
                            "self-harm/intent": false,
                            "sexual": false,
                            "sexual/minors": false,
                            "violence": false,
                            "violence/graphic": false
                        },
                        "category_scores": {
                            "harassment": 0.001,
                            "harassment/threatening": 0.0001,
                            "hate": 0.0001,
                            "hate/threatening": 0.0001,
                            "self-harm": 0.0001,
                            "self-harm/instructions": 0.0001,
                            "self-harm/intent": 0.0001,
                            "sexual": 0.0001,
                            "sexual/minors": 0.0001,
                            "violence": 0.0001,
                            "violence/graphic": 0.0001
                        }
                    }]
                }"#,
            )
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        let request = ModerationRequest::new("Hello, how are you?");

        let response = client.moderations().create(request).await.unwrap();
        assert_eq!(response.id, "modr-abc123");
        assert!(!response.results[0].flagged);
        mock.assert_async().await;
    }
}
