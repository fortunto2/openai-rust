// Runs resource — client.beta().threads().runs()

use super::BETA_HEADER;
use crate::client::OpenAI;
use crate::error::OpenAIError;
use crate::pagination::{Page, Paginator};
use crate::types::beta::{Run, RunCreateRequest, RunListParams, SubmitToolOutputsRequest};

/// Thread runs endpoint (beta).
pub struct Runs<'a> {
    client: &'a OpenAI,
    thread_id: String,
}

impl<'a> Runs<'a> {
    pub(crate) fn new(client: &'a OpenAI, thread_id: String) -> Self {
        Self { client, thread_id }
    }

    /// Create a run on a thread.
    ///
    /// `POST /threads/{thread_id}/runs`
    pub async fn create(&self, request: RunCreateRequest) -> Result<Run, OpenAIError> {
        let response = self
            .client
            .request(
                reqwest::Method::POST,
                &format!("/threads/{}/runs", self.thread_id),
            )
            .header(BETA_HEADER.0, BETA_HEADER.1)
            .json(&request)
            .send()
            .await?;
        OpenAI::handle_response(response).await
    }

    /// Retrieve a run.
    ///
    /// `GET /threads/{thread_id}/runs/{run_id}`
    pub async fn retrieve(&self, run_id: &str) -> Result<Run, OpenAIError> {
        let response = self
            .client
            .request(
                reqwest::Method::GET,
                &format!("/threads/{}/runs/{run_id}", self.thread_id),
            )
            .header(BETA_HEADER.0, BETA_HEADER.1)
            .send()
            .await?;
        OpenAI::handle_response(response).await
    }

    /// List runs for this thread.
    ///
    /// `GET /threads/{thread_id}/runs`
    pub async fn list(&self) -> Result<Vec<Run>, OpenAIError> {
        #[derive(serde::Deserialize)]
        struct RunList {
            data: Vec<Run>,
        }
        let response = self
            .client
            .request(
                reqwest::Method::GET,
                &format!("/threads/{}/runs", self.thread_id),
            )
            .header(BETA_HEADER.0, BETA_HEADER.1)
            .send()
            .await?;
        let list: RunList = OpenAI::handle_response(response).await?;
        Ok(list.data)
    }

    /// List runs with pagination parameters.
    ///
    /// `GET /threads/{thread_id}/runs`
    pub async fn list_page(&self, params: RunListParams) -> Result<Vec<Run>, OpenAIError> {
        #[derive(serde::Deserialize)]
        struct RunList {
            data: Vec<Run>,
        }
        let response = self
            .client
            .request(
                reqwest::Method::GET,
                &format!("/threads/{}/runs", self.thread_id),
            )
            .header(BETA_HEADER.0, BETA_HEADER.1)
            .query(&params.to_query())
            .send()
            .await?;
        let list: RunList = OpenAI::handle_response(response).await?;
        Ok(list.data)
    }

    /// Auto-paginate through all runs for this thread.
    pub fn list_auto(&self, params: RunListParams) -> Paginator<Run> {
        let client = self.client.clone();
        let thread_id = self.thread_id.clone();
        let base_params = params;
        Paginator::new(move |cursor| {
            let client = client.clone();
            let thread_id = thread_id.clone();
            let mut params = base_params.clone();
            if cursor.is_some() {
                params.after = cursor;
            }
            async move {
                #[derive(serde::Deserialize)]
                struct RunListResp {
                    data: Vec<Run>,
                    #[serde(default)]
                    has_more: Option<bool>,
                    #[serde(default)]
                    last_id: Option<String>,
                }
                let response = client
                    .request(reqwest::Method::GET, &format!("/threads/{thread_id}/runs"))
                    .header(BETA_HEADER.0, BETA_HEADER.1)
                    .query(&params.to_query())
                    .send()
                    .await?;
                let list: RunListResp = OpenAI::handle_response(response).await?;
                let after_cursor = list
                    .last_id
                    .clone()
                    .or_else(|| list.data.last().map(|r| r.id.clone()));
                Ok(Page {
                    has_more: list.has_more.unwrap_or(false),
                    after_cursor,
                    data: list.data,
                })
            }
        })
    }

    /// Cancel a run.
    ///
    /// `POST /threads/{thread_id}/runs/{run_id}/cancel`
    pub async fn cancel(&self, run_id: &str) -> Result<Run, OpenAIError> {
        let response = self
            .client
            .request(
                reqwest::Method::POST,
                &format!("/threads/{}/runs/{run_id}/cancel", self.thread_id),
            )
            .header(BETA_HEADER.0, BETA_HEADER.1)
            .send()
            .await?;
        OpenAI::handle_response(response).await
    }

    /// Submit tool outputs for a run.
    ///
    /// `POST /threads/{thread_id}/runs/{run_id}/submit_tool_outputs`
    pub async fn submit_tool_outputs(
        &self,
        run_id: &str,
        request: SubmitToolOutputsRequest,
    ) -> Result<Run, OpenAIError> {
        let response = self
            .client
            .request(
                reqwest::Method::POST,
                &format!(
                    "/threads/{}/runs/{run_id}/submit_tool_outputs",
                    self.thread_id
                ),
            )
            .header(BETA_HEADER.0, BETA_HEADER.1)
            .json(&request)
            .send()
            .await?;
        OpenAI::handle_response(response).await
    }
}

#[cfg(test)]
mod tests {
    use crate::OpenAI;
    use crate::config::ClientConfig;
    use crate::types::beta::RunCreateRequest;

    #[tokio::test]
    async fn test_runs_create() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/threads/thread_abc123/runs")
            .match_header("OpenAI-Beta", "assistants=v2")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "id": "run_abc123",
                    "object": "thread.run",
                    "created_at": 1699012949,
                    "thread_id": "thread_abc123",
                    "assistant_id": "asst_abc123",
                    "status": "queued",
                    "tools": []
                }"#,
            )
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        let request = RunCreateRequest::new("asst_abc123");

        let run = client
            .beta()
            .runs("thread_abc123")
            .create(request)
            .await
            .unwrap();
        assert_eq!(run.id, "run_abc123");
        assert_eq!(run.status, crate::types::beta::RunStatus::Queued);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_runs_cancel() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/threads/thread_abc123/runs/run_abc123/cancel")
            .match_header("OpenAI-Beta", "assistants=v2")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "id": "run_abc123",
                    "object": "thread.run",
                    "created_at": 1699012949,
                    "thread_id": "thread_abc123",
                    "assistant_id": "asst_abc123",
                    "status": "cancelling",
                    "tools": []
                }"#,
            )
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        let run = client
            .beta()
            .runs("thread_abc123")
            .cancel("run_abc123")
            .await
            .unwrap();
        assert_eq!(run.status, crate::types::beta::RunStatus::Cancelling);
        mock.assert_async().await;
    }
}
