import sys

with open('src/client.rs', 'r') as f:
    content = f.read()

target = '''    pub(crate) fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.config.base_url(), path);
        let mut req = self.http.request(method, &url);

        // Azure uses `api-key` header; standard OpenAI uses `Authorization: Bearer`
        if self.config.use_azure_api_key_header {
            req = req.header("api-key", &self.config.api_key());
        } else {
            req = req.bearer_auth(&self.config.api_key());
        }

        if let Some(ref org) = self.config.organization() {
            req = req.header("OpenAI-Organization", org);
        }
        if let Some(ref project) = self.config.project() {
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
    }'''

replacement = '''    pub(crate) fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
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

content = content.replace(target, replacement)

with open('src/client.rs', 'w') as f:
    f.write(content)

