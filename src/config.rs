// Client configuration

use std::env;

use reqwest::header::HeaderMap;

use crate::request_options::RequestOptions;

const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";
const DEFAULT_TIMEOUT_SECS: u64 = 600;
const DEFAULT_MAX_RETRIES: u32 = 2;

/// Configuration trait for API clients.
///
/// Allows implementing custom configurations for different providers
/// (e.g., standard OpenAI, Azure OpenAI, OpenRouter).
pub trait Config: Send + Sync + std::fmt::Debug {
    /// Returns the base URL for the API.
    fn base_url(&self) -> &str;

    /// Returns the API key.
    fn api_key(&self) -> &str;

    /// Hook to modify or add provider-specific headers and auth to a request.
    fn build_request(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder;

    /// Returns the organization ID, if any.
    fn organization(&self) -> Option<&str> {
        None
    }

    /// Returns the project ID, if any.
    fn project(&self) -> Option<&str> {
        None
    }

    /// Returns the timeout in seconds.
    fn timeout_secs(&self) -> u64 {
        DEFAULT_TIMEOUT_SECS
    }

    /// Returns the maximum number of retries.
    fn max_retries(&self) -> u32 {
        DEFAULT_MAX_RETRIES
    }

    /// Returns default headers to append to all requests.
    fn default_headers(&self) -> Option<&HeaderMap> {
        None
    }

    /// Returns default query parameters to append to all requests.
    fn default_query(&self) -> Option<&[(String, String)]> {
        None
    }

    /// Build the initial `RequestOptions` from config-level defaults.
    fn initial_options(&self) -> RequestOptions {
        let mut opts = RequestOptions::new();
        if let Some(h) = self.default_headers() {
            opts.headers = Some(h.clone());
        }
        if let Some(q) = self.default_query() {
            opts.query = Some(q.to_vec());
        }
        opts
    }
}

/// Configuration for the OpenAI client.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub api_key: String,
    pub base_url: String,
    pub organization: Option<String>,
    pub project: Option<String>,
    pub timeout_secs: u64,
    pub max_retries: u32,
    /// Default headers sent with every request.
    pub default_headers: Option<HeaderMap>,
    /// Default query parameters appended to every request URL.
    pub default_query: Option<Vec<(String, String)>>,
    /// When true, use `api-key` header instead of `Authorization: Bearer` for auth.
    /// This is used by Azure OpenAI deployments.
    pub(crate) use_azure_api_key_header: bool,
}

impl ClientConfig {
    /// Create a new config with the given API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: DEFAULT_BASE_URL.to_string(),
            organization: None,
            project: None,
            timeout_secs: DEFAULT_TIMEOUT_SECS,
            max_retries: DEFAULT_MAX_RETRIES,
            default_headers: None,
            default_query: None,
            use_azure_api_key_header: false,
        }
    }

    /// Create config from the `OPENAI_API_KEY` environment variable.
    pub fn from_env() -> Result<Self, crate::error::OpenAIError> {
        let api_key = env::var("OPENAI_API_KEY").map_err(|_| {
            crate::error::OpenAIError::InvalidArgument(
                "OPENAI_API_KEY environment variable not set".to_string(),
            )
        })?;
        Ok(Self::new(api_key))
    }

    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    pub fn organization(mut self, org: impl Into<String>) -> Self {
        self.organization = Some(org.into());
        self
    }

    pub fn project(mut self, project: impl Into<String>) -> Self {
        self.project = Some(project.into());
        self
    }

    pub fn timeout_secs(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    pub fn max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// Set default headers sent with every request.
    pub fn default_headers(mut self, headers: HeaderMap) -> Self {
        self.default_headers = Some(headers);
        self
    }

    /// Set default query parameters appended to every request URL.
    pub fn default_query(mut self, query: Vec<(String, String)>) -> Self {
        self.default_query = Some(query);
        self
    }

    /// Use Azure `api-key` header instead of `Authorization: Bearer` for auth.
    pub(crate) fn use_azure_api_key_header(mut self, enabled: bool) -> Self {
        self.use_azure_api_key_header = enabled;
        self
    }
}

impl Config for ClientConfig {
    fn base_url(&self) -> &str {
        &self.base_url
    }

    fn api_key(&self) -> &str {
        &self.api_key
    }

    fn build_request(&self, mut req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if self.use_azure_api_key_header {
            req = req.header("api-key", &self.api_key);
        } else {
            req = req.bearer_auth(&self.api_key);
        }

        if let Some(ref org) = self.organization {
            req = req.header("OpenAI-Organization", org);
        }
        if let Some(ref project) = self.project {
            req = req.header("OpenAI-Project", project);
        }

        req
    }

    fn organization(&self) -> Option<&str> {
        self.organization.as_deref()
    }

    fn project(&self) -> Option<&str> {
        self.project.as_deref()
    }

    fn timeout_secs(&self) -> u64 {
        self.timeout_secs
    }

    fn max_retries(&self) -> u32 {
        self.max_retries
    }

    fn default_headers(&self) -> Option<&HeaderMap> {
        self.default_headers.as_ref()
    }

    fn default_query(&self) -> Option<&[(String, String)]> {
        self.default_query.as_deref()
    }
}
