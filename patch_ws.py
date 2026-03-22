import sys

with open('src/websocket.rs', 'r') as f:
    content = f.read()

content = content.replace("use crate::config::ClientConfig;", "use crate::config::Config;")
content = content.replace("pub async fn connect(config: &ClientConfig) -> Result<Self, OpenAIError> {", "pub async fn connect(config: &dyn Config) -> Result<Self, OpenAIError> {")
content = content.replace("fn build_ws_url(config: &ClientConfig) -> String {", "fn build_ws_url(config: &dyn Config) -> String {")
content = content.replace("let base = &config.base_url;", "let base = config.base_url();")
content = content.replace("config.api_key", "config.api_key()")

with open('src/websocket.rs', 'w') as f:
    f.write(content)
