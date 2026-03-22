with open('src/rate_limit.rs', 'r') as f:
    content = f.read()

target = '''impl RateLimitTracker {
    pub fn new() -> Self {
        Self {
            info: RateLimitInfo::default(),
        }
    }'''

replacement = '''impl Default for RateLimitTracker {
    fn default() -> Self {
        Self {
            info: RateLimitInfo::default(),
        }
    }
}

impl RateLimitTracker {
    pub fn new() -> Self {
        Self::default()
    }'''

content = content.replace(target, replacement)
content = content.replace("use reqwest::{Response, Request, Method, Url};", "use reqwest::{Response, Request};")
content = content.replace("use super::*;", "")
content = content.replace('let mock = server.mock("GET", "/test")', 'let _mock = server.mock("GET", "/test")')

with open('src/rate_limit.rs', 'w') as f:
    f.write(content)
