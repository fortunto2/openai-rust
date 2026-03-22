import sys

with open('src/client.rs', 'r') as f:
    content = f.read()

content = content.replace("use crate::config::{ClientConfig, Config};", "use crate::config::{ClientConfig, Config};\nuse crate::middleware::Middleware;")

content = content.replace(
'''pub struct OpenAI {
    pub(crate) http: reqwest::Client,
    pub(crate) config: Arc<dyn Config>,
    pub(crate) options: RequestOptions,
}''',
'''pub struct OpenAI {
    pub(crate) http: reqwest::Client,
    pub(crate) config: Arc<dyn Config>,
    pub(crate) options: RequestOptions,
    pub(crate) middlewares: Vec<Arc<dyn Middleware>>,
}'''
)

content = content.replace(
'''        Self {
            http,
            config: Arc::new(config),
            options,
        }''',
'''        Self {
            http,
            config: Arc::new(config),
            options,
            middlewares: Vec::new(),
        }'''
)

content = content.replace(
'''    pub fn with_options(mut self, options: RequestOptions) -> Self {
        self.options = options;
        self
    }''',
'''    pub fn with_options(mut self, options: RequestOptions) -> Self {
        self.options = options;
        self
    }

    /// Add a middleware interceptor to the client.
    pub fn with_middleware<M: Middleware + 'static>(mut self, middleware: M) -> Self {
        self.middlewares.push(Arc::new(middleware));
        self
    }'''
)

# the handle_response area is tricky, let's inject middleware into the execute loop

with open('src/client.rs', 'w') as f:
    f.write(content)
