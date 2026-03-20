// Assistants resource — client.beta().assistants()

use super::BETA_HEADER;
use crate::client::OpenAI;
use crate::error::OpenAIError;
use crate::types::beta::{Assistant, AssistantCreateRequest, AssistantDeleted, AssistantList};

/// Access assistant endpoints (beta).
pub struct Assistants<'a> {
    client: &'a OpenAI,
}

impl<'a> Assistants<'a> {
    pub(crate) fn new(client: &'a OpenAI) -> Self {
        Self { client }
    }

    /// Create an assistant.
    ///
    /// `POST /assistants`
    pub async fn create(&self, request: AssistantCreateRequest) -> Result<Assistant, OpenAIError> {
        let response = self
            .client
            .request(reqwest::Method::POST, "/assistants")
            .header(BETA_HEADER.0, BETA_HEADER.1)
            .json(&request)
            .send()
            .await?;
        OpenAI::handle_response(response).await
    }

    /// List assistants.
    ///
    /// `GET /assistants`
    pub async fn list(&self) -> Result<AssistantList, OpenAIError> {
        let response = self
            .client
            .request(reqwest::Method::GET, "/assistants")
            .header(BETA_HEADER.0, BETA_HEADER.1)
            .send()
            .await?;
        OpenAI::handle_response(response).await
    }

    /// Retrieve an assistant.
    ///
    /// `GET /assistants/{assistant_id}`
    pub async fn retrieve(&self, assistant_id: &str) -> Result<Assistant, OpenAIError> {
        let response = self
            .client
            .request(reqwest::Method::GET, &format!("/assistants/{assistant_id}"))
            .header(BETA_HEADER.0, BETA_HEADER.1)
            .send()
            .await?;
        OpenAI::handle_response(response).await
    }

    /// Delete an assistant.
    ///
    /// `DELETE /assistants/{assistant_id}`
    pub async fn delete(&self, assistant_id: &str) -> Result<AssistantDeleted, OpenAIError> {
        let response = self
            .client
            .request(
                reqwest::Method::DELETE,
                &format!("/assistants/{assistant_id}"),
            )
            .header(BETA_HEADER.0, BETA_HEADER.1)
            .send()
            .await?;
        OpenAI::handle_response(response).await
    }
}

#[cfg(test)]
mod tests {
    use crate::OpenAI;
    use crate::config::ClientConfig;
    use crate::types::beta::AssistantCreateRequest;

    #[tokio::test]
    async fn test_assistants_create() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/assistants")
            .match_header("OpenAI-Beta", "assistants=v2")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "id": "asst_abc123",
                    "object": "assistant",
                    "created_at": 1699009709,
                    "model": "gpt-4o",
                    "name": "Math Tutor",
                    "tools": [{"type": "code_interpreter"}]
                }"#,
            )
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        let mut request = AssistantCreateRequest::new("gpt-4o");
        request.name = Some("Math Tutor".into());

        let assistant = client.beta().assistants().create(request).await.unwrap();
        assert_eq!(assistant.id, "asst_abc123");
        assert_eq!(assistant.name.as_deref(), Some("Math Tutor"));
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_assistants_delete() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("DELETE", "/assistants/asst_abc123")
            .match_header("OpenAI-Beta", "assistants=v2")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id": "asst_abc123", "object": "assistant.deleted", "deleted": true}"#)
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        let resp = client
            .beta()
            .assistants()
            .delete("asst_abc123")
            .await
            .unwrap();
        assert!(resp.deleted);
        mock.assert_async().await;
    }
}
