// WebSocket session for the Responses API.
//
// Provides persistent WebSocket connections to `wss://api.openai.com/v1/responses`
// for lower-latency, multi-turn interactions without per-request HTTP overhead.

use std::pin::Pin;
use std::task::{Context, Poll};

use futures_core::Stream;
use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

use crate::config::ClientConfig;
use crate::error::OpenAIError;
use crate::types::responses::{Response, ResponseCreateRequest, ResponseStreamEvent, ResponseTool};

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
}

impl WsSession {
    /// Connect to the OpenAI Responses WebSocket endpoint.
    ///
    /// Builds the WSS URL from the config's `base_url` (replacing `https://`
    /// with `wss://`) and appends `/responses`. Authentication is passed via
    /// the URL query parameter `?api_key=...`.
    pub async fn connect(config: &ClientConfig) -> Result<Self, OpenAIError> {
        let ws_url = build_ws_url(config);

        tracing::debug!(url = %ws_url, "connecting to WebSocket");

        let (stream, _response) = connect_async(&ws_url)
            .await
            .map_err(|e| OpenAIError::WebSocketError(format!("connection failed: {e}")))?;

        let (sink, reader) = stream.split();

        tracing::info!("WebSocket session connected");

        Ok(Self { sink, reader })
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
    async fn send_request(&mut self, request: &ResponseCreateRequest) -> Result<(), OpenAIError> {
        let text = serde_json::to_string(request)?;
        tracing::debug!(len = text.len(), "sending WS request");
        self.sink
            .send(Message::Text(text.into()))
            .await
            .map_err(|e| OpenAIError::WebSocketError(format!("send: {e}")))?;
        Ok(())
    }

    /// Read messages until a `response.completed` event is received.
    async fn read_until_completed(&mut self) -> Result<Response, OpenAIError> {
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

                    if event.type_ == "response.completed" {
                        let response: Response = serde_json::from_value(
                            event.data["response"].clone(),
                        )
                        .map_err(|e| {
                            OpenAIError::WebSocketError(format!(
                                "deserialize response.completed: {e}"
                            ))
                        })?;
                        return Ok(response);
                    }

                    if event.type_ == "response.failed" {
                        let message = event.data["response"]["error"]["message"]
                            .as_str()
                            .unwrap_or("unknown error")
                            .to_string();
                        let code = event.data["response"]["error"]["code"]
                            .as_str()
                            .map(String::from);
                        return Err(OpenAIError::ApiError {
                            status: 0,
                            message,
                            type_: Some("response_failed".into()),
                            code,
                        });
                    }

                    // Other events (deltas, created, etc.) are ignored in send()
                    tracing::trace!(event_type = %event.type_, "ws event (ignored in send)");
                }
                Message::Ping(data) => {
                    self.sink
                        .send(Message::Pong(data))
                        .await
                        .map_err(|e| OpenAIError::WebSocketError(format!("pong: {e}")))?;
                }
                Message::Close(_) => {
                    return Err(OpenAIError::WebSocketError(
                        "server closed connection".into(),
                    ));
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
                        if event.type_ == "response.completed" || event.type_ == "response.failed" {
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
/// base URL, appends `/responses`, and adds the API key as a query parameter.
fn build_ws_url(config: &ClientConfig) -> String {
    let base = &config.base_url;
    let ws_base = if base.starts_with("https://") {
        format!("wss://{}", &base["https://".len()..])
    } else if base.starts_with("http://") {
        format!("ws://{}", &base["http://".len()..])
    } else {
        base.clone()
    };

    format!("{ws_base}/responses?api_key={}", config.api_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_ws_url_default() {
        let config = ClientConfig::new("sk-test-key-123");
        let url = build_ws_url(&config);
        assert_eq!(
            url,
            "wss://api.openai.com/v1/responses?api_key=sk-test-key-123"
        );
    }

    #[test]
    fn test_build_ws_url_custom_base() {
        let config = ClientConfig::new("sk-abc").base_url("https://custom.api.com/v2");
        let url = build_ws_url(&config);
        assert_eq!(url, "wss://custom.api.com/v2/responses?api_key=sk-abc");
    }

    #[test]
    fn test_build_ws_url_http() {
        let config = ClientConfig::new("sk-local").base_url("http://localhost:8080/v1");
        let url = build_ws_url(&config);
        assert_eq!(url, "ws://localhost:8080/v1/responses?api_key=sk-local");
    }

    #[test]
    fn test_build_ws_url_no_scheme() {
        let config = ClientConfig::new("sk-x").base_url("wss://already-wss.com/v1");
        let url = build_ws_url(&config);
        assert_eq!(url, "wss://already-wss.com/v1/responses?api_key=sk-x");
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
