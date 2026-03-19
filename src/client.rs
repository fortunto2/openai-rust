// OpenAI client

use crate::config::ClientConfig;
use crate::error::{ErrorResponse, OpenAIError};
use crate::resources::chat::Chat;

/// The main OpenAI client.
#[derive(Debug, Clone)]
pub struct OpenAI {
    pub(crate) http: reqwest::Client,
    pub(crate) config: ClientConfig,
}

impl OpenAI {
    /// Create a new client with the given API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self::with_config(ClientConfig::new(api_key))
    }

    /// Create a client from a full config.
    pub fn with_config(config: ClientConfig) -> Self {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .expect("failed to build HTTP client");
        Self { http, config }
    }

    /// Create a client using the `OPENAI_API_KEY` environment variable.
    pub fn from_env() -> Result<Self, OpenAIError> {
        Ok(Self::with_config(ClientConfig::from_env()?))
    }

    /// Access the Chat resource.
    pub fn chat(&self) -> Chat<'_> {
        Chat::new(self)
    }

    /// Build a request with auth headers.
    pub(crate) fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.config.base_url, path);
        let mut req = self
            .http
            .request(method, &url)
            .bearer_auth(&self.config.api_key);

        if let Some(ref org) = self.config.organization {
            req = req.header("OpenAI-Organization", org);
        }
        if let Some(ref project) = self.config.project {
            req = req.header("OpenAI-Project", project);
        }

        req
    }

    /// Send a GET request and deserialize the response.
    pub(crate) async fn get<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
    ) -> Result<T, OpenAIError> {
        let response = self.request(reqwest::Method::GET, path).send().await?;
        Self::handle_response(response).await
    }

    /// Send a POST request with a JSON body and deserialize the response.
    pub(crate) async fn post<B: serde::Serialize, T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, OpenAIError> {
        let response = self
            .request(reqwest::Method::POST, path)
            .json(body)
            .send()
            .await?;
        Self::handle_response(response).await
    }

    /// Send a DELETE request and deserialize the response.
    pub(crate) async fn delete<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
    ) -> Result<T, OpenAIError> {
        let response = self.request(reqwest::Method::DELETE, path).send().await?;
        Self::handle_response(response).await
    }

    /// Handle API response: check status, parse errors or deserialize body.
    async fn handle_response<T: serde::de::DeserializeOwned>(
        response: reqwest::Response,
    ) -> Result<T, OpenAIError> {
        let status = response.status();
        if status.is_success() {
            let body = response.text().await?;
            let value: T = serde_json::from_str(&body)?;
            Ok(value)
        } else {
            let status_code = status.as_u16();
            let body = response.text().await.unwrap_or_default();
            if let Ok(error_resp) = serde_json::from_str::<ErrorResponse>(&body) {
                Err(OpenAIError::ApiError {
                    status: status_code,
                    message: error_resp.error.message,
                    type_: error_resp.error.type_,
                    code: error_resp.error.code,
                })
            } else {
                Err(OpenAIError::ApiError {
                    status: status_code,
                    message: body,
                    type_: None,
                    code: None,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_client() {
        let client = OpenAI::new("sk-test-key");
        assert_eq!(client.config.api_key, "sk-test-key");
        assert_eq!(client.config.base_url, "https://api.openai.com/v1");
    }

    #[test]
    fn test_with_config() {
        let config = ClientConfig::new("sk-test")
            .base_url("https://custom.api.com")
            .organization("org-123")
            .timeout_secs(30);
        let client = OpenAI::with_config(config);
        assert_eq!(client.config.base_url, "https://custom.api.com");
        assert_eq!(client.config.organization.as_deref(), Some("org-123"));
        assert_eq!(client.config.timeout_secs, 30);
    }

    #[tokio::test]
    async fn test_get_success() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/models/gpt-4")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{"id":"gpt-4","object":"model","created":1687882411,"owned_by":"openai"}"#,
            )
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));

        #[derive(serde::Deserialize)]
        struct Model {
            id: String,
            object: String,
        }

        let model: Model = client.get("/models/gpt-4").await.unwrap();
        assert_eq!(model.id, "gpt-4");
        assert_eq!(model.object, "model");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_post_success() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/chat/completions")
            .match_header("authorization", "Bearer sk-test")
            .match_header("content-type", "application/json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":"chatcmpl-123","object":"chat.completion"}"#)
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));

        #[derive(serde::Serialize)]
        struct Req {
            model: String,
        }
        #[derive(serde::Deserialize)]
        struct Resp {
            id: String,
        }

        let resp: Resp = client
            .post(
                "/chat/completions",
                &Req {
                    model: "gpt-4".into(),
                },
            )
            .await
            .unwrap();
        assert_eq!(resp.id, "chatcmpl-123");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_delete_success() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("DELETE", "/models/ft-abc")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":"ft-abc","deleted":true}"#)
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));

        #[derive(serde::Deserialize)]
        struct DeleteResp {
            id: String,
            deleted: bool,
        }

        let resp: DeleteResp = client.delete("/models/ft-abc").await.unwrap();
        assert_eq!(resp.id, "ft-abc");
        assert!(resp.deleted);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_api_error_response() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/models/nonexistent")
            .with_status(404)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{"error":{"message":"The model 'nonexistent' does not exist","type":"invalid_request_error","param":null,"code":"model_not_found"}}"#,
            )
            .create_async()
            .await;

        let client = OpenAI::with_config(ClientConfig::new("sk-test").base_url(server.url()));

        #[derive(Debug, serde::Deserialize)]
        struct Model {
            _id: String,
        }

        let err = client
            .get::<Model>("/models/nonexistent")
            .await
            .unwrap_err();
        match err {
            OpenAIError::ApiError {
                status,
                message,
                type_,
                code,
            } => {
                assert_eq!(status, 404);
                assert!(message.contains("does not exist"));
                assert_eq!(type_.as_deref(), Some("invalid_request_error"));
                assert_eq!(code.as_deref(), Some("model_not_found"));
            }
            other => panic!("expected ApiError, got: {other:?}"),
        }
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_auth_headers() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/test")
            .match_header("authorization", "Bearer sk-key")
            .match_header("OpenAI-Organization", "org-abc")
            .match_header("OpenAI-Project", "proj-xyz")
            .with_status(200)
            .with_body(r#"{"ok":true}"#)
            .create_async()
            .await;

        let client = OpenAI::with_config(
            ClientConfig::new("sk-key")
                .base_url(server.url())
                .organization("org-abc")
                .project("proj-xyz"),
        );

        #[derive(serde::Deserialize)]
        struct Resp {
            ok: bool,
        }

        let resp: Resp = client.get("/test").await.unwrap();
        assert!(resp.ok);
        mock.assert_async().await;
    }
}
