// Test the rate limit parsing
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    #[tokio::test]
    async fn test_rate_limit_middleware() {
        use reqwest::{Response, Request, Method, Url};
        use crate::middleware::Middleware;
        use crate::rate_limit::RateLimitTracker;
        
        let tracker = RateLimitTracker::new();
        
        let mut server = mockito::Server::new_async().await;
        let mock = server.mock("GET", "/test")
            .with_header("x-ratelimit-limit-requests", "500")
            .with_header("x-ratelimit-remaining-tokens", "9990")
            .with_header("x-ratelimit-reset-requests", "1m20s")
            .with_status(200)
            .create_async().await;
            
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
