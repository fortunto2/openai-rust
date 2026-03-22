use crate::middleware::Middleware;
use reqwest::{Request, Response};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Rate limit information extracted from API responses.
#[derive(Debug, Clone, Default)]
pub struct RateLimitInfo {
    pub limit_requests: Arc<AtomicU32>,
    pub limit_tokens: Arc<AtomicU32>,
    pub remaining_requests: Arc<AtomicU32>,
    pub remaining_tokens: Arc<AtomicU32>,
    pub reset_requests_ms: Arc<AtomicU64>,
    pub reset_tokens_ms: Arc<AtomicU64>,
}

/// Middleware that extracts rate limit headers from API responses and updates a shared `RateLimitInfo`.
#[derive(Debug, Clone, Default)]
pub struct RateLimitTracker {
    pub info: RateLimitInfo,
}

impl RateLimitTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn info(&self) -> RateLimitInfo {
        self.info.clone()
    }

    fn parse_reset_ms(value: &str) -> Option<u64> {
        // x-ratelimit-reset is typically formatted as "1s", "100ms", "1m20s"
        let value = value.trim();
        let mut ms = 0;

        let mut current_num = 0;
        let mut chars = value.chars().peekable();

        while let Some(c) = chars.next() {
            if c.is_ascii_digit() {
                current_num = current_num * 10 + c.to_digit(10).unwrap() as u64;
            } else if c == 'm' {
                if chars.peek() == Some(&'s') {
                    chars.next();
                    ms += current_num;
                } else {
                    ms += current_num * 60 * 1000;
                }
                current_num = 0;
            } else if c == 's' {
                ms += current_num * 1000;
                current_num = 0;
            } else if c == 'h' {
                ms += current_num * 60 * 60 * 1000;
                current_num = 0;
            }
        }

        Some(ms)
    }
}

#[async_trait::async_trait]
impl Middleware for RateLimitTracker {
    async fn on_request(&self, _req: &mut Request) -> Result<(), crate::error::OpenAIError> {
        Ok(())
    }

    async fn on_response(&self, res: &Response) -> Result<(), crate::error::OpenAIError> {
        let headers = res.headers();

        if let Some(val) = headers
            .get("x-ratelimit-limit-requests")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())
        {
            self.info.limit_requests.store(val, Ordering::SeqCst);
        }
        if let Some(val) = headers
            .get("x-ratelimit-limit-tokens")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())
        {
            self.info.limit_tokens.store(val, Ordering::SeqCst);
        }
        if let Some(val) = headers
            .get("x-ratelimit-remaining-requests")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())
        {
            self.info.remaining_requests.store(val, Ordering::SeqCst);
        }
        if let Some(val) = headers
            .get("x-ratelimit-remaining-tokens")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())
        {
            self.info.remaining_tokens.store(val, Ordering::SeqCst);
        }

        if let Some(val) = headers
            .get("x-ratelimit-reset-requests")
            .and_then(|v| v.to_str().ok())
            .and_then(Self::parse_reset_ms)
        {
            self.info.reset_requests_ms.store(val, Ordering::SeqCst);
        }
        if let Some(val) = headers
            .get("x-ratelimit-reset-tokens")
            .and_then(|v| v.to_str().ok())
            .and_then(Self::parse_reset_ms)
        {
            self.info.reset_tokens_ms.store(val, Ordering::SeqCst);
        }

        Ok(())
    }
}
// Test the rate limit parsing
#[cfg(test)]
mod tests {

    use std::sync::atomic::Ordering;

    #[tokio::test]
    async fn test_rate_limit_middleware() {
        use crate::middleware::Middleware;
        use crate::rate_limit::RateLimitTracker;

        let tracker = RateLimitTracker::new();

        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/test")
            .with_header("x-ratelimit-limit-requests", "500")
            .with_header("x-ratelimit-remaining-tokens", "9990")
            .with_header("x-ratelimit-reset-requests", "1m20s")
            .with_status(200)
            .create_async()
            .await;

        crate::ensure_tls_provider();
        let client = reqwest::Client::new();
        let res = client.get(server.url() + "/test").send().await.unwrap();

        tracker.on_response(&res).await.unwrap();

        let info = tracker.info();
        assert_eq!(info.limit_requests.load(Ordering::SeqCst), 500);
        assert_eq!(info.remaining_tokens.load(Ordering::SeqCst), 9990);
        // 1m20s = 60s + 20s = 80s = 80000ms
        assert_eq!(info.reset_requests_ms.load(Ordering::SeqCst), 80000);
    }
}
