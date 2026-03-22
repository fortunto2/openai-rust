import sys

with open('src/client.rs', 'r') as f:
    content = f.read()

# Replace imports
content = content.replace("use crate::config::ClientConfig;", "use crate::config::{ClientConfig, Config};\nuse std::sync::Arc;")

# Replace struct field
content = content.replace("    pub(crate) config: ClientConfig,", "    pub(crate) config: Arc<dyn Config>,")

# Replace with_config method
content = content.replace(
'''    pub fn with_config(config: ClientConfig) -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        // Insert default headers if any from config
        if let Some(ref default_headers) = config.default_headers {
            for (key, value) in default_headers.iter() {
                headers.insert(key.clone(), value.clone());
            }
        }

        // Initialize default options
        let options = config.initial_options();

        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .default_headers(headers)
            .build()
            .unwrap_or_default();

        Self {
            http,
            config,
            options,
        }
    }''',
'''    pub fn with_config<C: Config + 'static>(config: C) -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        // Insert default headers if any from config
        if let Some(default_headers) = config.default_headers() {
            for (key, value) in default_headers.iter() {
                headers.insert(key.clone(), value.clone());
            }
        }

        // Initialize default options
        let options = config.initial_options();

        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs()))
            .default_headers(headers)
            .build()
            .unwrap_or_default();

        Self {
            http,
            config: Arc::new(config),
            options,
        }
    }'''
)

# Replace request method
content = content.replace(
'''    pub(crate) fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.config.base_url, path);
        let mut req = self.http.request(method, &url);

        // Azure uses `api-key` header; standard OpenAI uses `Authorization: Bearer`
        if self.config.use_azure_api_key_header {
            req = req.header("api-key", &self.config.api_key);
        } else {
            req = req.bearer_auth(&self.config.api_key);
        }

        if let Some(ref org) = self.config.organization {
            req = req.header("OpenAI-Organization", org);
        }
        if let Some(ref project) = self.config.project {
            req = req.header("OpenAI-Project", project);
        }

        // Apply client-level options
        if let Some(ref headers) = self.options.headers {
            for (key, value) in headers.iter() {
                req = req.header(key.clone(), value.clone());
            }
        }
        
        let mut merged_query = Vec::new();
        if let Some(ref q) = self.options.query {
            merged_query.extend(q.clone());
        }
        if !merged_query.is_empty() {
            req = req.query(&merged_query);
        }

        req
    }''',
'''    pub(crate) fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.config.base_url(), path);
        let mut req = self.http.request(method, &url);

        req = self.config.build_request(req);

        // Apply client-level options
        if let Some(ref headers) = self.options.headers {
            for (key, value) in headers.iter() {
                req = req.header(key.clone(), value.clone());
            }
        }
        
        let mut merged_query = Vec::new();
        if let Some(ref q) = self.options.query {
            merged_query.extend(q.clone());
        }
        if !merged_query.is_empty() {
            req = req.query(&merged_query);
        }

        req
    }'''
)

# Fix send loop max retries
content = content.replace("for _ in 0..=self.config.max_retries {", "for _ in 0..=self.config.max_retries() {")
content = content.replace("let max_retries = self.config.max_retries;", "let max_retries = self.config.max_retries();")

with open('src/client.rs', 'w') as f:
    f.write(content)
