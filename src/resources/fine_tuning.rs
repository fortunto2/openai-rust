// Fine-tuning resource — client.fine_tuning().jobs().create() / list() / etc.

use crate::client::OpenAI;
use crate::error::OpenAIError;
use crate::pagination::{Page, Paginator};
use crate::types::fine_tuning::{
    FineTuningEventListParams, FineTuningJob, FineTuningJobCreateRequest, FineTuningJobEvent,
    FineTuningJobEventList, FineTuningJobList, FineTuningJobListParams,
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

    /// List fine-tuning jobs with pagination parameters.
    ///
    /// `GET /fine_tuning/jobs`
    pub async fn list_page(
        &self,
        params: FineTuningJobListParams,
    ) -> Result<FineTuningJobList, OpenAIError> {
        self.client
            .get_with_query("/fine_tuning/jobs", &params.to_query())
            .await
    }

    /// Auto-paginate through all fine-tuning jobs.
    pub fn list_auto(&self, params: FineTuningJobListParams) -> Paginator<FineTuningJob> {
        let client = self.client.clone();
        let base_params = params;
        Paginator::new(move |cursor| {
            let client = client.clone();
            let mut params = base_params.clone();
            if cursor.is_some() {
                params.after = cursor;
            }
            async move {
                let list: FineTuningJobList = client
                    .get_with_query("/fine_tuning/jobs", &params.to_query())
                    .await?;
                let after_cursor = list.data.last().map(|j| j.id.clone());
                Ok(Page {
                    has_more: list.has_more.unwrap_or(false),
                    after_cursor,
                    data: list.data,
                })
            }
        })
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

    /// List events for a fine-tuning job with pagination parameters.
    ///
    /// `GET /fine_tuning/jobs/{job_id}/events`
    pub async fn list_events_page(
        &self,
        job_id: &str,
        params: FineTuningEventListParams,
    ) -> Result<FineTuningJobEventList, OpenAIError> {
        self.client
            .get_with_query(
                &format!("/fine_tuning/jobs/{job_id}/events"),
                &params.to_query(),
            )
            .await
    }

    /// Auto-paginate through all events for a fine-tuning job.
    pub fn list_events_auto(
        &self,
        job_id: &str,
        params: FineTuningEventListParams,
    ) -> Paginator<FineTuningJobEvent> {
        let client = self.client.clone();
        let job_id = job_id.to_string();
        let base_params = params;
        Paginator::new(move |cursor| {
            let client = client.clone();
            let job_id = job_id.clone();
            let mut params = base_params.clone();
            if cursor.is_some() {
                params.after = cursor;
            }
            async move {
                let path = format!("/fine_tuning/jobs/{job_id}/events");
                let list: FineTuningJobEventList =
                    client.get_with_query(&path, &params.to_query()).await?;
                let after_cursor = list.data.last().map(|e| e.id.clone());
                Ok(Page {
                    has_more: list.has_more.unwrap_or(false),
                    after_cursor,
                    data: list.data,
                })
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::OpenAI;
    use crate::config::ClientConfig;
    use crate::types::fine_tuning::FineTuningJobCreateRequest;

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
        assert_eq!(
            job.status,
            crate::types::fine_tuning::FineTuningStatus::Running
        );
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
        assert_eq!(
            job.status,
            crate::types::fine_tuning::FineTuningStatus::Cancelled
        );
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
        assert_eq!(
            events.data[0].level,
            crate::types::fine_tuning::FineTuningEventLevel::Info
        );
        mock.assert_async().await;
    }
}
