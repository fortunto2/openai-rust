// Files resource — client.files().create() / list() / retrieve() / delete() / content()

use crate::client::OpenAI;
use crate::error::OpenAIError;
use crate::types::file::{FileDeleted, FileList, FileObject, FileUploadParams};

/// Access file endpoints.
pub struct Files<'a> {
    client: &'a OpenAI,
}

impl<'a> Files<'a> {
    pub(crate) fn new(client: &'a OpenAI) -> Self {
        Self { client }
    }

    /// Upload a file.
    ///
    /// `POST /files`
    pub async fn create(&self, params: FileUploadParams) -> Result<FileObject, OpenAIError> {
        let form = reqwest::multipart::Form::new()
            .part(
                "file",
                reqwest::multipart::Part::bytes(params.file).file_name(params.filename),
            )
            .text("purpose", params.purpose);

        self.client.post_multipart("/files", form).await
    }

    /// List files.
    ///
    /// `GET /files`
    pub async fn list(&self) -> Result<FileList, OpenAIError> {
        self.client.get("/files").await
    }

    /// Retrieve a file by ID.
    ///
    /// `GET /files/{file_id}`
    pub async fn retrieve(&self, file_id: &str) -> Result<FileObject, OpenAIError> {
        self.client.get(&format!("/files/{file_id}")).await
    }

    /// Delete a file.
    ///
    /// `DELETE /files/{file_id}`
    pub async fn delete(&self, file_id: &str) -> Result<FileDeleted, OpenAIError> {
        self.client.delete(&format!("/files/{file_id}")).await
    }

    /// Retrieve file content as bytes.
    ///
    /// `GET /files/{file_id}/content`
    pub async fn content(&self, file_id: &str) -> Result<bytes::Bytes, OpenAIError> {
        self.client
            .get_raw(&format!("/files/{file_id}/content"))
            .await
    }
}

#[cfg(test)]
mod tests {
    use crate::OpenAI;
    use crate::config::ClientConfig;
    use crate::types::file::FileUploadParams;

    #[tokio::test]
    async fn test_files_create() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/files")
            .match_header("authorization", "Bearer sk-test")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "id": "file-abc123",
                    "object": "file",
                    "bytes": 120000,
                    "created_at": 1677610602,
                    "filename": "data.jsonl",
                    "purpose": "fine-tune",
                    "status": "uploaded"
                }"#,
            )
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        let params = FileUploadParams::new(b"test data".to_vec(), "data.jsonl", "fine-tune");

        let response = client.files().create(params).await.unwrap();
        assert_eq!(response.id, "file-abc123");
        assert_eq!(response.purpose, "fine-tune");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_files_list() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/files")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "object": "list",
                    "data": [{
                        "id": "file-abc123",
                        "object": "file",
                        "bytes": 120000,
                        "created_at": 1677610602,
                        "filename": "data.jsonl",
                        "purpose": "fine-tune",
                        "status": "processed"
                    }]
                }"#,
            )
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        let response = client.files().list().await.unwrap();
        assert_eq!(response.data.len(), 1);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_files_retrieve() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/files/file-abc123")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "id": "file-abc123",
                    "object": "file",
                    "bytes": 120000,
                    "created_at": 1677610602,
                    "filename": "data.jsonl",
                    "purpose": "fine-tune",
                    "status": "processed"
                }"#,
            )
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        let file = client.files().retrieve("file-abc123").await.unwrap();
        assert_eq!(file.id, "file-abc123");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_files_delete() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("DELETE", "/files/file-abc123")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id": "file-abc123", "object": "file", "deleted": true}"#)
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        let resp = client.files().delete("file-abc123").await.unwrap();
        assert!(resp.deleted);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_files_content() {
        let mut server = mockito::Server::new_async().await;
        let content_bytes = b"line1\nline2\nline3";
        let mock = server
            .mock("GET", "/files/file-abc123/content")
            .with_status(200)
            .with_header("content-type", "application/octet-stream")
            .with_body(content_bytes)
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        let response = client.files().content("file-abc123").await.unwrap();
        assert_eq!(response.as_ref(), content_bytes);
        mock.assert_async().await;
    }
}
