#!/bin/bash
sed -i '' 's/use crate::config::ClientConfig;/use crate::config::{ClientConfig, Config};\nuse std::sync::Arc;/g' src/client.rs
sed -i '' 's/pub(crate) config: ClientConfig,/pub(crate) config: Arc<dyn Config>,/g' src/client.rs
sed -i '' 's/pub fn with_config(config: ClientConfig) -> Self/pub fn with_config<C: Config + '\'static\''>(config: C) -> Self/g' src/client.rs
sed -i '' 's/if let Some(ref default_headers) = config.default_headers {/if let Some(default_headers) = config.default_headers() {/g' src/client.rs
sed -i '' 's/\.timeout(std::time::Duration::from_secs(config.timeout_secs))/\.timeout(std::time::Duration::from_secs(config.timeout_secs()))/g' src/client.rs
sed -i '' 's/config,/config: Arc::new(config),/g' src/client.rs

# the request block
sed -i '' 's/let url = format!("{}{}", self.config.base_url, path);/let url = format!("{}{}", self.config.base_url(), path);/g' src/client.rs
sed -i '' '/if self.config.use_azure_api_key_header {/,/req = req.header("OpenAI-Project", project);/d' src/client.rs
sed -i '' '/let mut req = self.http.request(method, &url);/a\
\
        req = self.config.build_request(req);
' src/client.rs
sed -i '' '/\/\/ Azure uses `api-key` header; standard OpenAI uses `Authorization: Bearer`/d' src/client.rs
sed -i '' '/if let Some(ref org) = self.config.organization {/d' src/client.rs
sed -i '' '/if let Some(ref project) = self.config.project {/d' src/client.rs
sed -i '' '/} else {/d' src/client.rs

sed -i '' 's/for _ in 0..=self.config.max_retries {/for _ in 0..=self.config.max_retries() {/g' src/client.rs
sed -i '' 's/let max_retries = self.config.max_retries;/let max_retries = self.config.max_retries();/g' src/client.rs
