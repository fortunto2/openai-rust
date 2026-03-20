// Uploads resource — client.uploads().create() / cancel() / complete()

use crate::client::OpenAI;
use crate::error::OpenAIError;
use crate::types::upload::{Upload, UploadCompleteRequest, UploadCreateRequest};

/// Access upload endpoints.
pub struct Uploads<'a> {
    client: &'a OpenAI,
}

impl<'a> Uploads<'a> {
    pub(crate) fn new(client: &'a OpenAI) -> Self {
        Self { client }
    }

    /// Create an upload.
    ///
    /// `POST /uploads`
    pub async fn create(&self, request: UploadCreateRequest) -> Result<Upload, OpenAIError> {
        self.client.post("/uploads", &request).await
    }

    /// Cancel an upload.
    ///
    /// `POST /uploads/{upload_id}/cancel`
    pub async fn cancel(&self, upload_id: &str) -> Result<Upload, OpenAIError> {
        self.client
            .post(
                &format!("/uploads/{upload_id}/cancel"),
                &serde_json::Value::Null,
            )
            .await
    }

    /// Complete an upload with part IDs.
    ///
    /// `POST /uploads/{upload_id}/complete`
    pub async fn complete(
        &self,
        upload_id: &str,
        request: UploadCompleteRequest,
    ) -> Result<Upload, OpenAIError> {
        self.client
            .post(&format!("/uploads/{upload_id}/complete"), &request)
            .await
    }
}

#[cfg(test)]
mod tests {
    use crate::OpenAI;
    use crate::config::ClientConfig;
    use crate::types::upload::UploadCreateRequest;

    #[tokio::test]
    async fn test_uploads_create() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/uploads")
            .match_header("authorization", "Bearer sk-test")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "id": "upload_abc123",
                    "object": "upload",
                    "bytes": 2000000,
                    "filename": "data.jsonl",
                    "purpose": "fine-tune",
                    "status": "pending",
                    "created_at": 1699012949,
                    "expires_at": 1699016549
                }"#,
            )
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        let request = UploadCreateRequest::new(
            2_000_000,
            "data.jsonl",
            "text/jsonl",
            crate::types::file::FilePurpose::FineTune,
        );

        let upload = client.uploads().create(request).await.unwrap();
        assert_eq!(upload.id, "upload_abc123");
        assert_eq!(upload.status, crate::types::upload::UploadStatus::Pending);
        mock.assert_async().await;
    }
}
