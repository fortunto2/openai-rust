// Threads resource — client.beta().threads()

use super::BETA_HEADER;
use crate::client::OpenAI;
use crate::error::OpenAIError;
use crate::pagination::{Page, Paginator};
use crate::types::beta::{
    Message, MessageCreateRequest, MessageList, MessageListParams, Thread, ThreadCreateRequest,
    ThreadDeleted,
};

/// Access thread endpoints (beta).
pub struct Threads<'a> {
    client: &'a OpenAI,
}

impl<'a> Threads<'a> {
    pub(crate) fn new(client: &'a OpenAI) -> Self {
        Self { client }
    }

    /// Create a thread.
    ///
    /// `POST /threads`
    pub async fn create(&self, request: ThreadCreateRequest) -> Result<Thread, OpenAIError> {
        let response = self
            .client
            .request(reqwest::Method::POST, "/threads")
            .header(BETA_HEADER.0, BETA_HEADER.1)
            .json(&request)
            .send()
            .await?;
        OpenAI::handle_response(response).await
    }

    /// Retrieve a thread.
    ///
    /// `GET /threads/{thread_id}`
    pub async fn retrieve(&self, thread_id: &str) -> Result<Thread, OpenAIError> {
        let response = self
            .client
            .request(reqwest::Method::GET, &format!("/threads/{thread_id}"))
            .header(BETA_HEADER.0, BETA_HEADER.1)
            .send()
            .await?;
        OpenAI::handle_response(response).await
    }

    /// Delete a thread.
    ///
    /// `DELETE /threads/{thread_id}`
    pub async fn delete(&self, thread_id: &str) -> Result<ThreadDeleted, OpenAIError> {
        let response = self
            .client
            .request(reqwest::Method::DELETE, &format!("/threads/{thread_id}"))
            .header(BETA_HEADER.0, BETA_HEADER.1)
            .send()
            .await?;
        OpenAI::handle_response(response).await
    }

    /// Access messages sub-resource.
    pub fn messages(&self, thread_id: &str) -> Messages<'_> {
        Messages {
            client: self.client,
            thread_id: thread_id.to_string(),
        }
    }
}

/// Thread messages sub-resource.
pub struct Messages<'a> {
    client: &'a OpenAI,
    thread_id: String,
}

impl<'a> Messages<'a> {
    /// Create a message in a thread.
    ///
    /// `POST /threads/{thread_id}/messages`
    pub async fn create(&self, request: MessageCreateRequest) -> Result<Message, OpenAIError> {
        let response = self
            .client
            .request(
                reqwest::Method::POST,
                &format!("/threads/{}/messages", self.thread_id),
            )
            .header(BETA_HEADER.0, BETA_HEADER.1)
            .json(&request)
            .send()
            .await?;
        OpenAI::handle_response(response).await
    }

    /// List messages in a thread.
    ///
    /// `GET /threads/{thread_id}/messages`
    pub async fn list(&self) -> Result<MessageList, OpenAIError> {
        let response = self
            .client
            .request(
                reqwest::Method::GET,
                &format!("/threads/{}/messages", self.thread_id),
            )
            .header(BETA_HEADER.0, BETA_HEADER.1)
            .send()
            .await?;
        OpenAI::handle_response(response).await
    }

    /// List messages with pagination parameters.
    ///
    /// `GET /threads/{thread_id}/messages`
    pub async fn list_page(&self, params: MessageListParams) -> Result<MessageList, OpenAIError> {
        let response = self
            .client
            .request(
                reqwest::Method::GET,
                &format!("/threads/{}/messages", self.thread_id),
            )
            .header(BETA_HEADER.0, BETA_HEADER.1)
            .query(&params.to_query())
            .send()
            .await?;
        OpenAI::handle_response(response).await
    }

    /// Auto-paginate through all messages in a thread.
    pub fn list_auto(&self, params: MessageListParams) -> Paginator<Message> {
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
                let response = client
                    .request(
                        reqwest::Method::GET,
                        &format!("/threads/{thread_id}/messages"),
                    )
                    .header(BETA_HEADER.0, BETA_HEADER.1)
                    .query(&params.to_query())
                    .send()
                    .await?;
                let list: MessageList = OpenAI::handle_response(response).await?;
                let after_cursor = list
                    .last_id
                    .clone()
                    .or_else(|| list.data.last().map(|m| m.id.clone()));
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
    use crate::types::beta::ThreadCreateRequest;

    #[tokio::test]
    async fn test_threads_create() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/threads")
            .match_header("OpenAI-Beta", "assistants=v2")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "id": "thread_abc123",
                    "object": "thread",
                    "created_at": 1699012949
                }"#,
            )
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        let thread = client
            .beta()
            .threads()
            .create(ThreadCreateRequest::default())
            .await
            .unwrap();
        assert_eq!(thread.id, "thread_abc123");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_threads_delete() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("DELETE", "/threads/thread_abc123")
            .match_header("OpenAI-Beta", "assistants=v2")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id": "thread_abc123", "object": "thread.deleted", "deleted": true}"#)
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));
        let resp = client
            .beta()
            .threads()
            .delete("thread_abc123")
            .await
            .unwrap();
        assert!(resp.deleted);
        mock.assert_async().await;
    }
}
