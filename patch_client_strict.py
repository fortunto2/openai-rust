import sys

with open('src/client.rs', 'r') as f:
    content = f.read()

# exact replacements
content = content.replace("pub struct OpenAI {\n    pub(crate) http: reqwest::Client,\n    pub(crate) config: ClientConfig,\n    pub(crate) options: RequestOptions,\n}", "pub struct OpenAI {\n    pub(crate) http: reqwest::Client,\n    pub(crate) config: std::sync::Arc<dyn crate::config::Config>,\n    pub(crate) options: RequestOptions,\n}")

content = content.replace("pub fn with_config(config: ClientConfig) -> Self {", "pub fn with_config<C: crate::config::Config + 'static>(config: C) -> Self {")
content = content.replace("if let Some(ref default_headers) = config.default_headers {", "if let Some(default_headers) = config.default_headers() {")
content = content.replace(".timeout(std::time::Duration::from_secs(config.timeout_secs))", ".timeout(std::time::Duration::from_secs(config.timeout_secs()))")
content = content.replace("        Self {\n            http,\n            config,\n            options,\n        }", "        Self {\n            http,\n            config: std::sync::Arc::new(config),\n            options,\n        }")

# block replace request
target_req = """    pub(crate) fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
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
        }"""
replacement_req = """    pub(crate) fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.config.base_url(), path);
        let req = self.http.request(method, &url);
        let mut req = self.config.build_request(req);"""
content = content.replace(target_req, replacement_req)

content = content.replace("for _ in 0..=self.config.max_retries {", "for _ in 0..=self.config.max_retries() {")
content = content.replace("let max_retries = self.config.max_retries;", "let max_retries = self.config.max_retries();")

with open('src/client.rs', 'w') as f:
    f.write(content)
