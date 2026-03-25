// WebSocket session for the Responses API.
//
// Provides persistent WebSocket connections to `wss://api.openai.com/v1/responses`
// for lower-latency, multi-turn interactions without per-request HTTP overhead.

use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use futures_core::Stream;
use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async_with_config};

use crate::config::Config;
use crate::error::OpenAIError;
use crate::types::responses::{Response, ResponseCreateRequest, ResponseStreamEvent, ResponseTool};

/// Default timeout for `read_until_completed` (5 minutes).
const DEFAULT_WS_RESPONSE_TIMEOUT: Duration = Duration::from_secs(300);

type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;
type WsSink = SplitSink<WsStream, Message>;
type WsReader = futures_util::stream::SplitStream<WsStream>;

/// A persistent WebSocket session to the OpenAI Responses API.
///
/// Holds an open connection to `wss://api.openai.com/v1/responses` for
/// low-latency, multi-turn request/response flows without per-request
/// HTTP overhead.
///
/// # Usage
///
/// ```no_run
/// # use openai_oxide::{OpenAI, types::responses::*};
/// # async fn example() -> Result<(), openai_oxide::OpenAIError> {
/// let client = OpenAI::from_env()?;
/// let mut session = client.ws_session().await?;
///
/// // Send a request and get the full response
/// let request = ResponseCreateRequest::new("gpt-4o-mini")
///     .input("Hello, world!");
/// let response = session.send(request).await?;
/// println!("{}", response.output_text());
///
/// // Multi-turn: send a follow-up using the same session
/// let follow_up = ResponseCreateRequest::new("gpt-4o-mini")
///     .input("Tell me more")
///     .previous_response_id(&response.id);
/// let response2 = session.send(follow_up).await?;
///
/// session.close().await?;
/// # Ok(())
/// # }
/// ```
pub struct WsSession {
    sink: WsSink,
    reader: WsReader,
    response_timeout: Duration,
}

impl WsSession {
    /// Connect to the OpenAI Responses WebSocket endpoint.
    ///
    /// Builds the WSS URL from the config's `base_url` (replacing `https://`
    /// with `wss://`) and appends `/responses`. Authentication is via
    /// `Authorization: Bearer` header.
    pub async fn connect(config: &dyn Config) -> Result<Self, OpenAIError> {
        #[cfg(not(target_arch = "wasm32"))]
        crate::ensure_tls_provider();

        let ws_url = build_ws_url(config);

        tracing::debug!(url = %ws_url, "connecting to WebSocket");

        // Build request with Authorization header
        let request = tokio_tungstenite::tungstenite::http::Request::builder()
            .uri(&ws_url)
            .header("Authorization", format!("Bearer {}", config.api_key()))
            .header("Sec-WebSocket-Version", "13")
            .header(
                "Sec-WebSocket-Key",
                tokio_tungstenite::tungstenite::handshake::client::generate_key(),
            )
            .header(
                "Host",
                reqwest::Url::parse(&ws_url)
                    .map(|u| u.host_str().unwrap_or("api.openai.com").to_string())
                    .unwrap_or_else(|_| "api.openai.com".to_string()),
            )
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .body(())
            .map_err(|e| OpenAIError::WebSocketError(format!("build request: {e}")))?;

        let (stream, _response) = connect_async_with_config(request, None, false)
            .await
            .map_err(|e| OpenAIError::WebSocketError(format!("connection failed: {e}")))?;

        let (sink, reader) = stream.split();

        tracing::info!("WebSocket session connected");
        let response_timeout = DEFAULT_WS_RESPONSE_TIMEOUT;

        Ok(Self {
            sink,
            reader,
            response_timeout,
        })
    }

    /// Set the timeout for waiting on `response.completed`.
    ///
    /// Default is 5 minutes. Set to a lower value for latency-sensitive use cases.
    pub fn with_response_timeout(mut self, timeout: Duration) -> Self {
        self.response_timeout = timeout;
        self
    }

    /// Send a request and wait for the complete `Response`.
    ///
    /// Serializes the request as JSON, sends it over the WebSocket, then reads
    /// events until a `response.completed` event arrives containing the full
    /// [`Response`] object.
    ///
    /// Only one in-flight request is supported at a time (no multiplexing).
    pub async fn send(&mut self, request: ResponseCreateRequest) -> Result<Response, OpenAIError> {
        self.send_request(&request).await?;
        self.read_until_completed().await
    }

    /// Send a request and return a stream of events.
    ///
    /// Events are yielded as they arrive from the server, including deltas
    /// and the final `response.completed` event.
    pub async fn send_stream(
        &mut self,
        request: ResponseCreateRequest,
    ) -> Result<WsEventStream<'_>, OpenAIError> {
        self.send_request(&request).await?;
        Ok(WsEventStream {
            reader: &mut self.reader,
            done: false,
        })
    }

    /// Pre-load tools and instructions without generating a response.
    ///
    /// Sends a "warmup" request that loads tools and instructions into the
    /// session cache server-side, reducing latency on subsequent requests.
    /// The server processes this but does not generate model output.
    pub async fn warmup(
        &mut self,
        model: impl Into<String>,
        tools: Option<Vec<ResponseTool>>,
        instructions: Option<String>,
    ) -> Result<(), OpenAIError> {
        // Build a minimal request with the tools/instructions but no input
        let mut warmup_body = serde_json::json!({
            "model": model.into(),
        });

        if let Some(tools) = tools {
            warmup_body["tools"] = serde_json::to_value(&tools)
                .map_err(|e| OpenAIError::WebSocketError(format!("serialize tools: {e}")))?;
        }
        if let Some(instructions) = instructions {
            warmup_body["instructions"] = serde_json::Value::String(instructions);
        }

        let text = serde_json::to_string(&warmup_body)?;
        self.sink
            .send(Message::Text(text.into()))
            .await
            .map_err(|e| OpenAIError::WebSocketError(format!("send warmup: {e}")))?;

        // Read until we get response.completed for the warmup
        let _response = self.read_until_completed().await?;

        Ok(())
    }

    /// Close the WebSocket connection gracefully.
    pub async fn close(mut self) -> Result<(), OpenAIError> {
        self.sink
            .send(Message::Close(None))
            .await
            .map_err(|e| OpenAIError::WebSocketError(format!("close: {e}")))?;
        Ok(())
    }

    /// Send a serialized request over the WebSocket.
    ///
    /// Wraps the request body with `"type": "response.create"` as required
    /// by the OpenAI WebSocket protocol.
    async fn send_request(&mut self, request: &ResponseCreateRequest) -> Result<(), OpenAIError> {
        // WebSocket API expects: {"type": "response.create", ...request_fields}
        let mut value = serde_json::to_value(request)?;
        if let serde_json::Value::Object(ref mut map) = value {
            map.insert(
                "type".to_string(),
                serde_json::Value::String("response.create".to_string()),
            );

            // WORKAROUND: OpenAI WS bug — decimal temperature (0.7, 1.2) causes
            // silent close=1000. Remove non-integer temperature from WS requests.
            // HTTP is unaffected. Tracking: https://community.openai.com/t/1375536
            if let Some(serde_json::Value::Number(n)) = map.get("temperature") {
                if let Some(f) = n.as_f64() {
                    if f.fract() != 0.0 {
                        tracing::debug!(
                            temperature = f,
                            "stripping decimal temperature (OpenAI WS bug)"
                        );
                        map.remove("temperature");
                    }
                }
            }
        }
        let text = serde_json::to_string(&value)?;
        tracing::debug!(len = text.len(), "sending WS request");
        tracing::trace!(body = %text, "WS request body");
        self.sink
            .send(Message::Text(text.into()))
            .await
            .map_err(|e| OpenAIError::WebSocketError(format!("send: {e}")))?;
        Ok(())
    }

    /// Read messages until a `response.completed` event is received.
    ///
    /// Returns an error if `response_timeout` elapses before completion.
    async fn read_until_completed(&mut self) -> Result<Response, OpenAIError> {
        tokio::time::timeout(self.response_timeout, self.read_until_completed_inner())
            .await
            .map_err(|_| {
                OpenAIError::WebSocketError(format!(
                    "timed out waiting for response.completed after {:?}",
                    self.response_timeout
                ))
            })?
    }

    /// Inner loop that reads until `response.completed`.
    async fn read_until_completed_inner(&mut self) -> Result<Response, OpenAIError> {
        loop {
            let msg = self
                .reader
                .next()
                .await
                .ok_or_else(|| {
                    OpenAIError::WebSocketError(
                        "connection closed before response.completed".into(),
                    )
                })?
                .map_err(|e| OpenAIError::WebSocketError(format!("read: {e}")))?;

            match msg {
                Message::Text(text) => {
                    let event: ResponseStreamEvent = serde_json::from_str(&text)?;

                    match event {
                        ResponseStreamEvent::ResponseCompleted(evt) => {
                            return Ok(evt.response);
                        }
                        ResponseStreamEvent::ResponseFailed(evt) => {
                            let message = evt
                                .response
                                .error
                                .as_ref()
                                .map(|e| e.message.clone())
                                .unwrap_or_else(|| "unknown error".into());
                            let code = evt.response.error.as_ref().map(|e| e.code.clone());
                            return Err(OpenAIError::ApiError {
                                status: 0,
                                message,
                                type_: Some("response_failed".into()),
                                code,
                                request_id: None,
                            });
                        }
                        other => {
                            tracing::trace!(event_type = %other.event_type(), "ws event (ignored in send)");
                        }
                    }
                }
                Message::Ping(data) => {
                    self.sink
                        .send(Message::Pong(data))
                        .await
                        .map_err(|e| OpenAIError::WebSocketError(format!("pong: {e}")))?;
                }
                Message::Close(frame) => {
                    let reason = frame
                        .as_ref()
                        .map(|f| format!("code={}, reason={}", f.code, f.reason))
                        .unwrap_or_else(|| "no close frame".into());
                    tracing::warn!(reason = %reason, "WS server closed connection");
                    return Err(OpenAIError::WebSocketError(format!(
                        "server closed connection: {reason}"
                    )));
                }
                _ => {}
            }
        }
    }
}

/// A stream of WebSocket events from a single response.
///
/// Yields [`ResponseStreamEvent`] items until the `response.completed` event
/// is received, at which point the stream ends.
pub struct WsEventStream<'a> {
    reader: &'a mut WsReader,
    done: bool,
}

impl<'a> Stream for WsEventStream<'a> {
    type Item = Result<ResponseStreamEvent, OpenAIError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        if this.done {
            return Poll::Ready(None);
        }

        match this.reader.poll_next_unpin(cx) {
            Poll::Ready(Some(Ok(msg))) => match msg {
                Message::Text(text) => match serde_json::from_str::<ResponseStreamEvent>(&text) {
                    Ok(event) => {
                        if matches!(
                            event,
                            ResponseStreamEvent::ResponseCompleted(_)
                                | ResponseStreamEvent::ResponseFailed(_)
                        ) {
                            this.done = true;
                        }
                        Poll::Ready(Some(Ok(event)))
                    }
                    Err(e) => Poll::Ready(Some(Err(OpenAIError::JsonError(e)))),
                },
                Message::Close(_) => {
                    this.done = true;
                    Poll::Ready(None)
                }
                Message::Ping(_) => {
                    // Pong is handled by tungstenite automatically in most cases,
                    // but we need to wake to continue polling
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
                _ => {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            },
            Poll::Ready(Some(Err(e))) => {
                this.done = true;
                Poll::Ready(Some(Err(OpenAIError::WebSocketError(format!("read: {e}")))))
            }
            Poll::Ready(None) => {
                this.done = true;
                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Build the WebSocket URL from a `ClientConfig`.
///
/// Replaces `https://` with `wss://` (or `http://` with `ws://`) in the
/// base URL and appends `/responses`. Auth is via headers, not query params.
fn build_ws_url(config: &dyn Config) -> String {
    let base = config.base_url();
    let ws_base = if base.starts_with("https://") {
        format!("wss://{}", &base["https://".len()..])
    } else if base.starts_with("http://") {
        format!("ws://{}", &base["http://".len()..])
    } else {
        base.to_string()
    };

    format!("{ws_base}/responses")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ClientConfig;

    #[test]
    fn test_build_ws_url_default() {
        let config = ClientConfig::new("sk-test-key-123");
        let url = build_ws_url(&config);
        assert_eq!(url, "wss://api.openai.com/v1/responses");
    }

    #[test]
    fn test_build_ws_url_custom_base() {
        let config = ClientConfig::new("sk-abc").base_url("https://custom.api.com/v2");
        let url = build_ws_url(&config);
        assert_eq!(url, "wss://custom.api.com/v2/responses");
    }

    #[test]
    fn test_build_ws_url_http() {
        let config = ClientConfig::new("sk-local").base_url("http://localhost:8080/v1");
        let url = build_ws_url(&config);
        assert_eq!(url, "ws://localhost:8080/v1/responses");
    }

    #[test]
    fn test_build_ws_url_no_scheme() {
        let config = ClientConfig::new("sk-x").base_url("wss://already-wss.com/v1");
        let url = build_ws_url(&config);
        assert_eq!(url, "wss://already-wss.com/v1/responses");
    }

    /// Live WS test — requires OPENAI_API_KEY.
    /// Run: OPENAI_API_KEY=sk-... cargo test -p openai-oxide ws_live -- --ignored
    #[tokio::test]
    #[ignore]
    async fn ws_live() {
        let client = crate::OpenAI::from_env().expect("OPENAI_API_KEY");
        eprintln!("Connecting WS...");
        let mut session = WsSession::connect(&*client.config)
            .await
            .expect("ws connect failed");
        eprintln!("Connected. Sending request...");
        let req = ResponseCreateRequest::new("gpt-5.4-mini").input("Say hello in exactly 3 words");
        let resp = session.send(req).await.expect("ws send failed");
        let text = resp.output_text();
        eprintln!("Response: {text}");
        assert!(!text.is_empty(), "Expected non-empty response");
        session.close().await.ok();
    }

    /// WS with large payload — test if big messages cause close=1000.
    /// Run: OPENAI_API_KEY=sk-... cargo test -p openai-oxide --features websocket ws_live_large -- --ignored --nocapture
    #[tokio::test]
    #[ignore]
    async fn ws_live_large() {
        let client = crate::OpenAI::from_env().expect("OPENAI_API_KEY");
        let mut session = WsSession::connect(&*client.config)
            .await
            .expect("ws connect");
        // Build a ~70KB payload similar to coaching context
        let big_system = "X".repeat(60_000);
        let req = ResponseCreateRequest::new("gpt-5.4-mini")
            .instructions(&big_system)
            .input("Say hi in 3 words")
            .max_output_tokens(50);
        eprintln!("Sending ~70KB request via WS...");
        match session.send(req).await {
            Ok(resp) => eprintln!("OK with large payload: {}", resp.output_text()),
            Err(e) => panic!("FAILED with large payload: {e}"),
        }
    }

    /// WS with tools — test function calling format.
    /// Run: OPENAI_API_KEY=sk-... cargo test -p openai-oxide --features websocket ws_live_tools -- --ignored --nocapture
    #[tokio::test]
    #[ignore]
    async fn ws_live_tools() {
        let client = crate::OpenAI::from_env().expect("OPENAI_API_KEY");
        let mut session = WsSession::connect(&*client.config)
            .await
            .expect("ws connect");
        let req = ResponseCreateRequest::new("gpt-5.4-mini")
            .input("What is 2+2?")
            .tools(vec![ResponseTool::Function {
                name: "calculate".into(),
                description: Some("Math calculation".into()),
                parameters: Some(serde_json::json!({"type":"object","properties":{"expr":{"type":"string"}},"required":["expr"]})),
                strict: None,
            }])
            .store(true);
        eprintln!("Sending WS request with tools...");
        match session.send(req).await {
            Ok(resp) => {
                let fcs = resp.function_calls();
                eprintln!(
                    "OK tools: {} function calls, text={}",
                    fcs.len(),
                    resp.output_text()
                );
            }
            Err(e) => panic!("FAILED with tools: {e}"),
        }
    }

    /// WS simulating exact souffleur-server payload: large instructions + tools + store + Items input.
    /// Run: OPENAI_API_KEY=sk-... cargo test -p openai-oxide --features websocket ws_live_server_sim -- --ignored --nocapture
    #[tokio::test]
    #[ignore]
    async fn ws_live_server_sim() {
        let client = crate::OpenAI::from_env().expect("OPENAI_API_KEY");
        let mut session = WsSession::connect(&*client.config)
            .await
            .expect("ws connect");

        // Simulate souffleur coach payload: Items input + tools + store
        let big_system = "You are a sales coach. ".repeat(2000); // ~44KB
        let input = vec![
            serde_json::json!({"type": "message", "role": "system", "content": big_system}),
            serde_json::json!({"type": "message", "role": "user", "content": "Rep said hello to customer"}),
        ];
        let tools = vec![ResponseTool::Function {
            name: "whisper".into(),
            description: Some("Coach the rep".into()),
            parameters: Some(serde_json::json!({
                "type": "object",
                "properties": {"message": {"type": "string"}},
                "required": ["message"]
            })),
            strict: None,
        }];
        let mut req = ResponseCreateRequest::new("gpt-5.4-mini");
        req.input = Some(crate::types::responses::ResponseInput::Items(input));
        req = req.tools(tools).store(true).max_output_tokens(100);

        let payload = serde_json::to_string(&req).unwrap();
        eprintln!(
            "Sending server-sim payload via WS ({} bytes)...",
            payload.len()
        );
        match session.send(req).await {
            Ok(resp) => {
                let fcs = resp.function_calls();
                eprintln!(
                    "OK: {} function_calls, text={}",
                    fcs.len(),
                    resp.output_text()
                );
            }
            Err(e) => panic!("FAILED server-sim: {e}"),
        }
    }

    /// WS with delay — test idle tolerance.
    /// Run: OPENAI_API_KEY=sk-... cargo test -p openai-oxide --features websocket ws_live_delay -- --ignored --nocapture
    #[tokio::test]
    #[ignore]
    async fn ws_live_delay() {
        let client = crate::OpenAI::from_env().expect("OPENAI_API_KEY");
        let mut session = WsSession::connect(&*client.config)
            .await
            .expect("ws connect");
        eprintln!("Connected. Waiting 5 seconds...");
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        eprintln!("Sending after 5s delay...");
        let req = ResponseCreateRequest::new("gpt-5.4-mini").input("Say hi");
        match session.send(req).await {
            Ok(resp) => eprintln!("OK after delay: {}", resp.output_text()),
            Err(e) => panic!("FAILED after 5s delay: {e}"),
        }
    }

    #[test]
    fn test_request_serialization_for_ws() {
        let request = ResponseCreateRequest::new("gpt-4o-mini")
            .input("Hello")
            .instructions("Be concise")
            .temperature(0.5);

        let json = serde_json::to_string(&request).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["model"], "gpt-4o-mini");
        assert_eq!(parsed["input"], "Hello");
        assert_eq!(parsed["instructions"], "Be concise");
        assert_eq!(parsed["temperature"], 0.5);
        // stream should not be set (WS doesn't use it)
        assert!(parsed.get("stream").is_none());
    }
}
