// Batches resource — client.batches().create() / list() / retrieve() / cancel()

use crate::client::OpenAI;
use crate::error::OpenAIError;
use crate::types::batch::{Batch, BatchCreateRequest, BatchList};

/// Access batch endpoints.
pub struct Batches<'a> {
    client: &'a OpenAI,
}

impl<'a> Batches<'a> {
    pub(crate) fn new(client: &'a OpenAI) -> Self {
        Self { client }
    }

    /// Create a batch.
    ///
    /// `POST /batches`
    pub async fn create(&self, request: BatchCreateRequest) -> Result<Batch, OpenAIError> {
        self.client.post("/batches", &request).await
    }

    /// List batches.
    ///
    /// `GET /batches`
    pub async fn list(&self) -> Result<BatchList, OpenAIError> {
        self.client.get("/batches").await
    }

    /// Retrieve a batch.
    ///
    /// `GET /batches/{batch_id}`
    pub async fn retrieve(&self, batch_id: &str) -> Result<Batch, OpenAIError> {
        self.client.get(&format!("/batches/{batch_id}")).await
    }

    /// Cancel a batch.
    ///
    /// `POST /batches/{batch_id}/cancel`
    pub async fn cancel(&self, batch_id: &str) -> Result<Batch, OpenAIError> {
        self.client
            .post(
                &format!("/batches/{batch_id}/cancel"),
                &serde_json::Value::Null,
            )
            .await
    }
}

#[cfg(test)]
mod tests {
    use crate::OpenAI;
    use crate::config::ClientConfig;
    use crate::types::batch::BatchCreateRequest;

    const BATCH_JSON: &str = r#"{
        "id": "batch_abc123",
        "object": "batch",
        "endpoint": "/v1/chat/completions",
        "input_file_id": "file-abc123",
        "completion_window": "24h",
        "status": "validating",
        "created_at": 1699012949
    }"#;

    #[tokio::test]
    async fn test_batches_create() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/batches")
            .match_header("authorization", "Bearer sk-test")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(BATCH_JSON)
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        let request = BatchCreateRequest::new("file-abc123", "/v1/chat/completions", "24h");

        let batch = client.batches().create(request).await.unwrap();
        assert_eq!(batch.id, "batch_abc123");
        assert_eq!(batch.status, "validating");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_batches_retrieve() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/batches/batch_abc123")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(BATCH_JSON)
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        let batch = client.batches().retrieve("batch_abc123").await.unwrap();
        assert_eq!(batch.id, "batch_abc123");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_batches_cancel() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/batches/batch_abc123/cancel")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "id": "batch_abc123",
                    "object": "batch",
                    "endpoint": "/v1/chat/completions",
                    "input_file_id": "file-abc123",
                    "completion_window": "24h",
                    "status": "cancelling",
                    "created_at": 1699012949
                }"#,
            )
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        let batch = client.batches().cancel("batch_abc123").await.unwrap();
        assert_eq!(batch.status, "cancelling");
        mock.assert_async().await;
    }
}
