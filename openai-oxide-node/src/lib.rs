#![deny(clippy::all)]

use futures_util::StreamExt;
use napi::bindgen_prelude::*;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;
use openai_oxide::types::responses::ResponseCreateRequest;
use openai_oxide::websocket::WsSession;
use openai_oxide::OpenAI;
use std::sync::Arc;
use tokio::sync::Mutex;

#[napi]
pub struct Client {
    inner: OpenAI,
}

#[napi]
pub struct NodeWsSession {
    inner: Arc<Mutex<Option<WsSession>>>,
}

#[napi]
impl Client {
    #[napi(constructor)]
    pub fn new() -> Result<Self> {
        let inner = OpenAI::from_env().map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(Self { inner })
    }

    #[napi(
        ts_args_type = "request: Record<string, any>",
        ts_return_type = "Promise<Record<string, any>>"
    )]
    pub async fn create_chat_completion(
        &self,
        request: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let res = self
            .inner
            .chat()
            .completions()
            .create_raw(&request)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(res)
    }

    #[napi(
        ts_args_type = "request: Record<string, any>, tsfn: (err: Error | null, event: Record<string, any> | null) => void",
        ts_return_type = "Promise<void>"
    )]
    pub async fn create_chat_stream(
        &self,
        request: serde_json::Value,
        tsfn: ThreadsafeFunction<serde_json::Value>,
    ) -> Result<()> {
        let mut body = request;

        // Ensure stream=true
        if let Some(obj) = body.as_object_mut() {
            obj.insert("stream".to_string(), serde_json::Value::Bool(true));
            // Force stream_options for usage tracking
            if !obj.contains_key("stream_options") {
                obj.insert(
                    "stream_options".to_string(),
                    serde_json::json!({"include_usage": true}),
                );
            }
        }

        match self
            .inner
            .chat()
            .completions()
            .create_stream_raw(&body)
            .await
        {
            Ok(mut stream) => {
                while let Some(item) = stream.next().await {
                    match item {
                        Ok(event) => {
                            tsfn.call(Ok(event), ThreadsafeFunctionCallMode::Blocking);
                        }
                        Err(e) => {
                            tsfn.call(
                                Err(Error::from_reason(e.to_string())),
                                ThreadsafeFunctionCallMode::Blocking,
                            );
                            break;
                        }
                    }
                }
                tsfn.call(
                    Ok(serde_json::json!({"type": "done"})),
                    ThreadsafeFunctionCallMode::Blocking,
                );
            }
            Err(e) => {
                tsfn.call(
                    Err(Error::from_reason(e.to_string())),
                    ThreadsafeFunctionCallMode::Blocking,
                );
            }
        }
        Ok(())
    }
    #[napi(
        ts_args_type = "request: Record<string, any>",
        ts_return_type = "Promise<Record<string, any>>"
    )]
    pub async fn create_response(&self, request: serde_json::Value) -> Result<serde_json::Value> {
        let res = self
            .inner
            .responses()
            .create_raw(&request)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(res)
    }

    #[napi(
        ts_args_type = "request: Record<string, any>, tsfn: (err: Error | null, event: Record<string, any> | null) => void",
        ts_return_type = "Promise<void>"
    )]
    pub async fn create_stream(
        &self,
        request: serde_json::Value,
        tsfn: ThreadsafeFunction<serde_json::Value>,
    ) -> Result<()> {
        let mut body = request;

        // Ensure stream=true
        if let Some(obj) = body.as_object_mut() {
            obj.insert("stream".to_string(), serde_json::Value::Bool(true));
        }

        match self.inner.responses().create_stream_raw(&body).await {
            Ok(mut stream) => {
                while let Some(item) = stream.next().await {
                    match item {
                        Ok(event) => {
                            tsfn.call(Ok(event), ThreadsafeFunctionCallMode::Blocking);
                        }
                        Err(e) => {
                            tsfn.call(
                                Err(Error::from_reason(e.to_string())),
                                ThreadsafeFunctionCallMode::Blocking,
                            );
                            break;
                        }
                    }
                }
                tsfn.call(
                    Ok(serde_json::json!({"type": "done"})),
                    ThreadsafeFunctionCallMode::Blocking,
                );
            }
            Err(e) => {
                tsfn.call(
                    Err(Error::from_reason(e.to_string())),
                    ThreadsafeFunctionCallMode::Blocking,
                );
            }
        }
        Ok(())
    }

    #[napi]
    pub async fn ws_session(&self) -> Result<NodeWsSession> {
        let session = self
            .inner
            .ws_session()
            .await
            .map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(NodeWsSession {
            inner: Arc::new(Mutex::new(Some(session))),
        })
    }
}

#[napi]
impl NodeWsSession {
    #[napi(
        ts_args_type = "model: string, input: string",
        ts_return_type = "Promise<Record<string, any>>"
    )]
    pub async fn send(&self, model: String, input: String) -> Result<serde_json::Value> {
        let mut lock = self.inner.lock().await;
        if let Some(session) = lock.as_mut() {
            let req = ResponseCreateRequest::new(model).input(input);
            let response = session
                .send(req)
                .await
                .map_err(|e| Error::from_reason(e.to_string()))?;
            return Ok(serde_json::to_value(response).unwrap_or(serde_json::Value::Null));
        }
        Err(Error::from_reason("Session closed"))
    }

    #[napi]
    pub async fn close(&self) -> Result<()> {
        let mut lock = self.inner.lock().await;
        if let Some(session) = lock.take() {
            let _ = session.close().await;
        }
        Ok(())
    }
}
