// Vector Stores resource — client.beta().vector_stores()

use super::BETA_HEADER;
use crate::client::OpenAI;
use crate::error::OpenAIError;
use crate::types::beta::{
    VectorStore, VectorStoreCreateRequest, VectorStoreDeleted, VectorStoreList,
};

/// Access vector store endpoints (beta).
pub struct VectorStores<'a> {
    client: &'a OpenAI,
}

impl<'a> VectorStores<'a> {
    pub(crate) fn new(client: &'a OpenAI) -> Self {
        Self { client }
    }

    /// Create a vector store.
    ///
    /// `POST /vector_stores`
    pub async fn create(
        &self,
        request: VectorStoreCreateRequest,
    ) -> Result<VectorStore, OpenAIError> {
        let response = self
            .client
            .request(reqwest::Method::POST, "/vector_stores")
            .header(BETA_HEADER.0, BETA_HEADER.1)
            .json(&request)
            .send()
            .await?;
        OpenAI::handle_response(response).await
    }

    /// List vector stores.
    ///
    /// `GET /vector_stores`
    pub async fn list(&self) -> Result<VectorStoreList, OpenAIError> {
        let response = self
            .client
            .request(reqwest::Method::GET, "/vector_stores")
            .header(BETA_HEADER.0, BETA_HEADER.1)
            .send()
            .await?;
        OpenAI::handle_response(response).await
    }

    /// Retrieve a vector store.
    ///
    /// `GET /vector_stores/{vector_store_id}`
    pub async fn retrieve(&self, vector_store_id: &str) -> Result<VectorStore, OpenAIError> {
        let response = self
            .client
            .request(
                reqwest::Method::GET,
                &format!("/vector_stores/{vector_store_id}"),
            )
            .header(BETA_HEADER.0, BETA_HEADER.1)
            .send()
            .await?;
        OpenAI::handle_response(response).await
    }

    /// Delete a vector store.
    ///
    /// `DELETE /vector_stores/{vector_store_id}`
    pub async fn delete(&self, vector_store_id: &str) -> Result<VectorStoreDeleted, OpenAIError> {
        let response = self
            .client
            .request(
                reqwest::Method::DELETE,
                &format!("/vector_stores/{vector_store_id}"),
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
    use crate::types::beta::VectorStoreCreateRequest;

    #[tokio::test]
    async fn test_vector_stores_create() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/vector_stores")
            .match_header("OpenAI-Beta", "assistants=v2")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "id": "vs_abc123",
                    "object": "vector_store",
                    "created_at": 1699012949,
                    "name": "My Store",
                    "status": "completed"
                }"#,
            )
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        let mut request = VectorStoreCreateRequest::default();
        request.name = Some("My Store".into());

        let vs = client.beta().vector_stores().create(request).await.unwrap();
        assert_eq!(vs.id, "vs_abc123");
        assert_eq!(vs.name.as_deref(), Some("My Store"));
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_vector_stores_delete() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("DELETE", "/vector_stores/vs_abc123")
            .match_header("OpenAI-Beta", "assistants=v2")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id": "vs_abc123", "object": "vector_store.deleted", "deleted": true}"#)
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        let resp = client
            .beta()
            .vector_stores()
            .delete("vs_abc123")
            .await
            .unwrap();
        assert!(resp.deleted);
        mock.assert_async().await;
    }
}
