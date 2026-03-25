// AI-TODO: consider migrating to `deadpool` crate for production-grade
// pool management (health checks, metrics, async recycle). Current impl
// is lightweight and OpenAI-specific — good enough for now.

//! WebSocket connection pool for the OpenAI Responses API.
//!
//! Reuses idle connections across consecutive WebSocket requests for
//! lower latency. Connections are keyed by `(url, api_key_hash)` so
//! different clients never share a socket. Stale entries are lazily
//! evicted on checkout — no background task needed.
//!
//! # Usage
//!
//! ```no_run
//! # use openai_oxide::ws_pool::{WsPool, PoolKey};
//! # async fn example() {
//! let pool = WsPool::shared();
//! let key = PoolKey::new("wss://api.openai.com/v1/responses", "sk-...");
//!
//! // Try to reuse an existing connection
//! if let Some((stream, created_at)) = pool.checkout(&key).await {
//!     // Use the pooled stream...
//! }
//!
//! // Return the connection when done
//! // pool.return_conn(key, stream, created_at).await;
//! # }
//! ```

use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::LazyLock;
use std::time::{Duration, Instant};

use tokio::sync::Mutex;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

/// Max idle connections per key (url + api_key).
const MAX_IDLE_PER_KEY: usize = 4;
/// Max idle connections total across all keys.
const MAX_IDLE_TOTAL: usize = 8;
/// Idle timeout — must be below OpenAI's 60s server-side timeout.
const IDLE_TIMEOUT: Duration = Duration::from_secs(55);
/// Rotate connections after this lifetime regardless of activity.
const MAX_LIFETIME: Duration = Duration::from_secs(300);

/// WebSocket stream type from tokio-tungstenite.
pub type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

/// Key for bucketing pooled connections by endpoint + credentials.
#[derive(Clone, Hash, Eq, PartialEq)]
pub struct PoolKey {
    url: String,
    key_hash: u64,
}

impl PoolKey {
    /// Create a pool key from WebSocket URL and API key.
    /// The API key is hashed immediately — never stored in plain text.
    pub fn new(url: &str, api_key: &str) -> Self {
        let mut hasher = DefaultHasher::new();
        api_key.hash(&mut hasher);
        Self {
            url: url.to_string(),
            key_hash: hasher.finish(),
        }
    }
}

struct IdleConnection {
    stream: WsStream,
    returned_at: Instant,
    created_at: Instant,
}

/// A bounded pool of idle WebSocket connections.
pub struct WsPool {
    connections: Mutex<HashMap<PoolKey, Vec<IdleConnection>>>,
}

impl WsPool {
    /// Access the global shared pool.
    pub fn shared() -> &'static Self {
        static POOL: LazyLock<WsPool> = LazyLock::new(WsPool::new);
        &POOL
    }

    fn new() -> Self {
        Self {
            connections: Mutex::new(HashMap::new()),
        }
    }

    /// Try to reclaim an idle connection for the given key.
    ///
    /// Returns `None` when the pool has nothing valid. Stale entries
    /// (idle timeout or max lifetime) are dropped automatically.
    pub async fn checkout(&self, key: &PoolKey) -> Option<(WsStream, Instant)> {
        let mut map = self.connections.lock().await;
        let bucket = map.get_mut(key)?;
        let now = Instant::now();

        while let Some(entry) = bucket.pop() {
            if now.duration_since(entry.returned_at) > IDLE_TIMEOUT {
                tracing::debug!("ws_pool: dropping idle-timeout connection");
                continue;
            }
            if now.duration_since(entry.created_at) > MAX_LIFETIME {
                tracing::debug!("ws_pool: dropping max-lifetime connection");
                continue;
            }
            if bucket.is_empty() {
                map.remove(key);
            }
            tracing::debug!("ws_pool: reusing pooled connection");
            return Some((entry.stream, entry.created_at));
        }

        map.remove(key);
        None
    }

    /// Return a still-healthy connection for future reuse.
    ///
    /// The connection is silently dropped if it exceeds `MAX_LIFETIME`
    /// or the pool is already at capacity.
    pub async fn return_conn(&self, key: PoolKey, stream: WsStream, created_at: Instant) {
        let now = Instant::now();
        if now.duration_since(created_at) > MAX_LIFETIME {
            tracing::debug!("ws_pool: not returning max-lifetime connection");
            return;
        }

        let mut map = self.connections.lock().await;

        let bucket = map.entry(key).or_default();
        if bucket.len() >= MAX_IDLE_PER_KEY {
            tracing::debug!("ws_pool: per-key cap reached, dropping oldest");
            bucket.remove(0);
        }
        bucket.push(IdleConnection {
            stream,
            returned_at: now,
            created_at,
        });

        let total: usize = map.values().map(Vec::len).sum();
        if total > MAX_IDLE_TOTAL {
            tracing::debug!("ws_pool: global cap reached, evicting oldest");
            Self::evict_oldest(&mut map);
        }
    }

    fn evict_oldest(map: &mut HashMap<PoolKey, Vec<IdleConnection>>) {
        let mut oldest_key: Option<PoolKey> = None;
        let mut oldest_idx: usize = 0;
        let mut oldest_time: Option<Instant> = None;

        for (key, bucket) in map.iter() {
            for (idx, entry) in bucket.iter().enumerate() {
                if oldest_time.is_none() || Some(entry.returned_at) < oldest_time {
                    oldest_key = Some(key.clone());
                    oldest_idx = idx;
                    oldest_time = Some(entry.returned_at);
                }
            }
        }

        if let Some(key) = oldest_key {
            if let Some(bucket) = map.get_mut(&key) {
                bucket.remove(oldest_idx);
                if bucket.is_empty() {
                    map.remove(&key);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::tungstenite::Message;

    type ServerWsStream = WebSocketStream<tokio::net::TcpStream>;

    async fn make_ws_pair() -> (WsStream, ServerWsStream) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let addr = listener.local_addr().expect("addr");

        let server_handle = tokio::spawn(async move {
            let (tcp, _) = listener.accept().await.expect("accept");
            tokio_tungstenite::accept_async(tcp)
                .await
                .expect("ws accept")
        });

        let url = format!("ws://127.0.0.1:{}", addr.port());
        let (client, _) = tokio_tungstenite::connect_async(&url)
            .await
            .expect("connect");
        let server = server_handle.await.expect("join server");
        (client, server)
    }

    fn test_key(url: &str, secret: &str) -> PoolKey {
        PoolKey::new(url, secret)
    }

    #[tokio::test]
    async fn checkout_empty_returns_none() {
        let pool = WsPool::new();
        let key = test_key("wss://api.openai.com/v1/responses", "sk-test");
        assert!(pool.checkout(&key).await.is_none());
    }

    #[tokio::test]
    async fn return_then_checkout() {
        let pool = WsPool::new();
        let key = test_key("wss://api.openai.com/v1/responses", "sk-test");
        let (client, mut server) = make_ws_pair().await;

        let created = Instant::now();
        pool.return_conn(key.clone(), client, created).await;

        let (mut stream, checkout_created) =
            pool.checkout(&key).await.expect("should get connection");
        assert_eq!(checkout_created, created);

        // Verify the connection still works.
        stream
            .send(Message::Text("hello".into()))
            .await
            .expect("send");
        let msg = server.next().await.expect("recv").expect("frame");
        assert_eq!(msg, Message::Text("hello".into()));

        assert!(pool.checkout(&key).await.is_none());
    }

    #[tokio::test]
    async fn idle_timeout_eviction() {
        let pool = WsPool::new();
        let key = test_key("wss://api.openai.com/v1/responses", "sk-test");
        let (client, _server) = make_ws_pair().await;

        let created = Instant::now();
        {
            let mut map = pool.connections.lock().await;
            map.entry(key.clone()).or_default().push(IdleConnection {
                stream: client,
                returned_at: created - (IDLE_TIMEOUT + Duration::from_secs(1)),
                created_at: created,
            });
        }

        assert!(pool.checkout(&key).await.is_none());
    }

    #[tokio::test]
    async fn max_lifetime_eviction() {
        let pool = WsPool::new();
        let key = test_key("wss://api.openai.com/v1/responses", "sk-test");
        let (client, _server) = make_ws_pair().await;

        let old_created = Instant::now() - (MAX_LIFETIME + Duration::from_secs(1));
        {
            let mut map = pool.connections.lock().await;
            map.entry(key.clone()).or_default().push(IdleConnection {
                stream: client,
                returned_at: Instant::now(),
                created_at: old_created,
            });
        }

        assert!(pool.checkout(&key).await.is_none());
    }

    #[tokio::test]
    async fn different_keys_isolated() {
        let pool = WsPool::new();
        let key_a = test_key("wss://api.openai.com/v1/responses", "sk-aaa");
        let key_b = test_key("wss://api.openai.com/v1/responses", "sk-bbb");

        let (client_a, _server_a) = make_ws_pair().await;
        pool.return_conn(key_a.clone(), client_a, Instant::now())
            .await;

        assert!(pool.checkout(&key_b).await.is_none());
        assert!(pool.checkout(&key_a).await.is_some());
    }

    #[tokio::test]
    async fn max_lifetime_rejected_on_return() {
        let pool = WsPool::new();
        let key = test_key("wss://api.openai.com/v1/responses", "sk-test");
        let (client, _server) = make_ws_pair().await;

        let old_created = Instant::now() - (MAX_LIFETIME + Duration::from_secs(1));
        pool.return_conn(key.clone(), client, old_created).await;

        let map = pool.connections.lock().await;
        assert!(map.get(&key).is_none());
    }
}
