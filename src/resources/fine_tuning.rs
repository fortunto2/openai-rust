// Fine-tuning resource — client.fine_tuning().jobs().create() / list() / etc.

use crate::client::OpenAI;
use crate::error::OpenAIError;
use crate::types::fine_tuning::{
    FineTuningJob, FineTuningJobCreateRequest, FineTuningJobEventList, FineTuningJobList,
};

/// Access fine-tuning endpoints.
pub struct FineTuning<'a> {
    client: &'a OpenAI,
}

impl<'a> FineTuning<'a> {
    pub(crate) fn new(client: &'a OpenAI) -> Self {
        Self { client }
    }

    /// Access fine-tuning jobs.
    pub fn jobs(&self) -> Jobs<'_> {
        Jobs {
            client: self.client,
        }
    }
}

/// Fine-tuning jobs endpoint.
pub struct Jobs<'a> {
    client: &'a OpenAI,
}

impl<'a> Jobs<'a> {
    /// Create a fine-tuning job.
    ///
    /// `POST /fine_tuning/jobs`
    pub async fn create(
        &self,
        request: FineTuningJobCreateRequest,
    ) -> Result<FineTuningJob, OpenAIError> {
        self.client.post("/fine_tuning/jobs", &request).await
    }

    /// List fine-tuning jobs.
    ///
    /// `GET /fine_tuning/jobs`
    pub async fn list(&self) -> Result<FineTuningJobList, OpenAIError> {
        self.client.get("/fine_tuning/jobs").await
    }

    /// Retrieve a fine-tuning job.
    ///
    /// `GET /fine_tuning/jobs/{job_id}`
    pub async fn retrieve(&self, job_id: &str) -> Result<FineTuningJob, OpenAIError> {
        self.client
            .get(&format!("/fine_tuning/jobs/{job_id}"))
            .await
    }

    /// Cancel a fine-tuning job.
    ///
    /// `POST /fine_tuning/jobs/{job_id}/cancel`
    pub async fn cancel(&self, job_id: &str) -> Result<FineTuningJob, OpenAIError> {
        self.client
            .post(
                &format!("/fine_tuning/jobs/{job_id}/cancel"),
                &serde_json::Value::Null,
            )
            .await
    }

    /// List events for a fine-tuning job.
    ///
    /// `GET /fine_tuning/jobs/{job_id}/events`
    pub async fn list_events(&self, job_id: &str) -> Result<FineTuningJobEventList, OpenAIError> {
        self.client
            .get(&format!("/fine_tuning/jobs/{job_id}/events"))
            .await
    }
}

#[cfg(test)]
mod tests {
    use crate::config::ClientConfig;
    use crate::types::fine_tuning::FineTuningJobCreateRequest;
    use crate::OpenAI;

    const JOB_JSON: &str = r#"{
        "id": "ftjob-abc123",
        "object": "fine_tuning.job",
        "created_at": 1677610602,
        "model": "gpt-4o-mini",
        "training_file": "file-abc123",
        "status": "running",
        "organization_id": "org-123",
        "result_files": [],
        "seed": 42
    }"#;

    #[tokio::test]
    async fn test_fine_tuning_jobs_create() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/fine_tuning/jobs")
            .match_header("authorization", "Bearer sk-test")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(JOB_JSON)
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        let request = FineTuningJobCreateRequest::new("gpt-4o-mini", "file-abc123");

        let job = client.fine_tuning().jobs().create(request).await.unwrap();
        assert_eq!(job.id, "ftjob-abc123");
        assert_eq!(job.status, "running");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_fine_tuning_jobs_list() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/fine_tuning/jobs")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(format!(
                r#"{{"object": "list", "data": [{}], "has_more": false}}"#,
                JOB_JSON
            ))
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        let response = client.fine_tuning().jobs().list().await.unwrap();
        assert_eq!(response.data.len(), 1);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_fine_tuning_jobs_cancel() {
        let mut server = mockito::Server::new_async().await;
        let cancelled_json = r#"{
            "id": "ftjob-abc123",
            "object": "fine_tuning.job",
            "created_at": 1677610602,
            "model": "gpt-4o-mini",
            "training_file": "file-abc123",
            "status": "cancelled",
            "organization_id": "org-123",
            "result_files": [],
            "seed": 42
        }"#;
        let mock = server
            .mock("POST", "/fine_tuning/jobs/ftjob-abc123/cancel")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(cancelled_json)
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        let job = client
            .fine_tuning()
            .jobs()
            .cancel("ftjob-abc123")
            .await
            .unwrap();
        assert_eq!(job.status, "cancelled");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_fine_tuning_jobs_list_events() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/fine_tuning/jobs/ftjob-abc123/events")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "object": "list",
                    "data": [{
                        "id": "ftevent-abc123",
                        "object": "fine_tuning.job.event",
                        "created_at": 1677610602,
                        "level": "info",
                        "message": "Training started"
                    }],
                    "has_more": false
                }"#,
            )
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        let events = client
            .fine_tuning()
            .jobs()
            .list_events("ftjob-abc123")
            .await
            .unwrap();
        assert_eq!(events.data.len(), 1);
        assert_eq!(events.data[0].level, "info");
        mock.assert_async().await;
    }
}
