// Embeddings resource — client.embeddings().create()

use crate::client::OpenAI;
use crate::error::OpenAIError;
use crate::types::embedding::{CreateEmbeddingResponse, EmbeddingRequest};

/// Access embedding endpoints.
pub struct Embeddings<'a> {
    client: &'a OpenAI,
}

impl<'a> Embeddings<'a> {
    pub(crate) fn new(client: &'a OpenAI) -> Self {
        Self { client }
    }

    /// Create embeddings.
    ///
    /// `POST /embeddings`
    pub async fn create(
        &self,
        request: EmbeddingRequest,
    ) -> Result<CreateEmbeddingResponse, OpenAIError> {
        self.client.post("/embeddings", &request).await
    }
}

#[cfg(test)]
mod tests {
    use crate::OpenAI;
    use crate::config::ClientConfig;
    use crate::types::embedding::EmbeddingRequest;

    #[tokio::test]
    async fn test_embeddings_create() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/embeddings")
            .match_header("authorization", "Bearer sk-test")
            .match_header("content-type", "application/json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "object": "list",
                    "data": [{
                        "object": "embedding",
                        "embedding": [0.0023, -0.0094, 0.0158],
                        "index": 0
                    }],
                    "model": "text-embedding-3-small",
                    "usage": {
                        "prompt_tokens": 8,
                        "total_tokens": 8
                    }
                }"#,
            )
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        let request = EmbeddingRequest::new("text-embedding-3-small", "Hello world");

        let response = client.embeddings().create(request).await.unwrap();
        assert_eq!(response.object, "list");
        assert_eq!(response.model, "text-embedding-3-small");
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].embedding.len(), 3);
        mock.assert_async().await;
    }
}
