// Models resource — client.models().list() / retrieve() / delete()

use crate::client::OpenAI;
use crate::error::OpenAIError;
use crate::types::model::{Model, ModelDeleted, ModelList};

/// Access model endpoints.
pub struct Models<'a> {
    client: &'a OpenAI,
}

impl<'a> Models<'a> {
    pub(crate) fn new(client: &'a OpenAI) -> Self {
        Self { client }
    }

    /// List available models.
    ///
    /// `GET /models`
    pub async fn list(&self) -> Result<ModelList, OpenAIError> {
        self.client.get("/models").await
    }

    /// Retrieve a model by ID.
    ///
    /// `GET /models/{model}`
    pub async fn retrieve(&self, model: &str) -> Result<Model, OpenAIError> {
        self.client.get(&format!("/models/{model}")).await
    }

    /// Delete a fine-tuned model.
    ///
    /// `DELETE /models/{model}`
    pub async fn delete(&self, model: &str) -> Result<ModelDeleted, OpenAIError> {
        self.client.delete(&format!("/models/{model}")).await
    }
}

#[cfg(test)]
mod tests {
    use crate::OpenAI;
    use crate::config::ClientConfig;

    #[tokio::test]
    async fn test_models_list() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/models")
            .match_header("authorization", "Bearer sk-test")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "object": "list",
                    "data": [
                        {"id": "gpt-4o", "object": "model", "created": 1687882411, "owned_by": "openai"},
                        {"id": "gpt-3.5-turbo", "object": "model", "created": 1677610602, "owned_by": "openai"}
                    ]
                }"#,
            )
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        let response = client.models().list().await.unwrap();
        assert_eq!(response.data.len(), 2);
        assert_eq!(response.data[0].id, "gpt-4o");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_models_retrieve() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/models/gpt-4o")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{"id": "gpt-4o", "object": "model", "created": 1687882411, "owned_by": "openai"}"#,
            )
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        let model = client.models().retrieve("gpt-4o").await.unwrap();
        assert_eq!(model.id, "gpt-4o");
        assert_eq!(model.owned_by, "openai");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_models_delete() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("DELETE", "/models/ft:gpt-4o:org:custom:id")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id": "ft:gpt-4o:org:custom:id", "object": "model", "deleted": true}"#)
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        let resp = client
            .models()
            .delete("ft:gpt-4o:org:custom:id")
            .await
            .unwrap();
        assert!(resp.deleted);
        mock.assert_async().await;
    }
}
