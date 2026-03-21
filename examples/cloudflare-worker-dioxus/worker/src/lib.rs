use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use worker::*;
use openai_oxide::types::responses::{ResponseCreateRequest, ResponseStreamEvent};

#[derive(Serialize, Deserialize, Debug)]
struct WsMessage {
    action: String,
    content: Option<String>,
}

#[event(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new();

    router
        .get_async("/api/ws", |req, ctx| async move {
            let namespace = ctx.env.durable_object("CHAT_DO")?;
            let id = namespace.id_from_name("global")?;
            let stub = id.get_stub()?;
            stub.fetch_with_request(req).await
        })
        .run(req, env)
        .await
}

#[durable_object]
pub struct ChatDurableObject {
    state: State,
    env: Env,
}

impl DurableObject for ChatDurableObject {
    fn new(state: State, env: Env) -> Self {
        Self { state, env }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        self.handle(req).await
    }
}

impl ChatDurableObject {
    async fn handle(&self, req: Request) -> Result<Response> {
        if req.headers().get("Upgrade")?.unwrap_or_default().to_lowercase() == "websocket" {
            let pair = WebSocketPair::new()?;
            let client = pair.client;
            let browser_ws = pair.server;
            browser_ws.accept()?;

            let url = req.url()?;
            let api_key = url.query_pairs().find(|(k, _)| k == "key").map(|(_, v)| v.into_owned())
                .filter(|k| !k.is_empty())
                .or_else(|| self.env.var("OPENAI_API_KEY").ok().map(|v| v.to_string()))
                .filter(|k| !k.is_empty());

            let api_key = match api_key {
                Some(k) => k,
                None => {
                    console_log!("OPENAI_API_KEY missing in query params and env");
                    browser_ws.close(Some(1011), Some("Missing API Key"))?;
                    return Response::from_websocket(client);
                }
            };

            // Connect to OpenAI Responses API via WebSocket
            let mut headers = Headers::new();
            headers.set("Upgrade", "websocket")?;
            headers.set("Authorization", &format!("Bearer {}", api_key))?;
            
            let mut init = RequestInit::new();
            init.with_method(Method::Get);
            init.with_headers(headers);

            let openai_req = Request::new_with_init("https://api.openai.com/v1/responses", &init)?;
            let openai_res = match Fetch::Request(openai_req).send().await {
                Ok(res) => res,
                Err(e) => {
                    console_log!("Failed to fetch OpenAI: {:?}", e);
                    browser_ws.close(Some(1011), Some("Failed to connect to OpenAI"))?;
                    return Response::from_websocket(client);
                }
            };

            let openai_status = openai_res.status_code();
            let openai_ws = match openai_res.websocket() {
                Some(ws) => ws,
                None => {
                    console_log!("OpenAI did not return a WebSocket. Status: {}", openai_status);
                    browser_ws.close(None, Some("Failed to connect to OpenAI"))?;
                    return Response::from_websocket(client);
                }
            };
            openai_ws.accept()?;

            wasm_bindgen_futures::spawn_local(async move {
                let mut browser_events = browser_ws.events().expect("Failed to get browser events");
                let mut openai_events = openai_ws.events().expect("Failed to get OpenAI events");

                loop {
                    tokio::select! {
                        Some(browser_event) = browser_events.next() => {
                            if let Ok(worker::WebsocketEvent::Message(msg)) = browser_event {
                                if let Some(text) = msg.text() {
                                    if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                                        if ws_msg.action == "send" {
                                            if let Some(content) = ws_msg.content {
                                                // Create a response.create event for OpenAI
                                                let o_req = ResponseCreateRequest::new("gpt-4o-mini").input(content);
                                                
                                                let mut value = serde_json::to_value(&o_req).unwrap();
                                                if let serde_json::Value::Object(ref mut map) = value {
                                                    map.insert("type".to_string(), serde_json::Value::String("response.create".to_string()));
                                                }
                                                let request_text = serde_json::to_string(&value).unwrap();
                                                
                                                let _ = openai_ws.send_with_str(request_text);
                                            }
                                        }
                                    }
                                }
                            } else if let Ok(worker::WebsocketEvent::Close(_)) = browser_event {
                                let _ = openai_ws.close(None, Some(""));
                                break;
                            }
                        }
                        Some(openai_event) = openai_events.next() => {
                            if let Ok(worker::WebsocketEvent::Message(msg)) = openai_event {
                                if let Some(text) = msg.text() {
                                    if let Ok(event) = serde_json::from_str::<ResponseStreamEvent>(&text) {
                                        if event.type_ == "response.output_text.delta" {
                                            if let Some(delta) = event.data["delta"].as_str() {
                                                let reply = WsMessage {
                                                    action: "chunk".into(),
                                                    content: Some(delta.to_string()),
                                                };
                                                if let Ok(json) = serde_json::to_string(&reply) {
                                                    let _ = browser_ws.send_with_str(json);
                                                }
                                            }
                                        } else if event.type_ == "response.completed" {
                                            let done_msg = WsMessage {
                                                action: "done".into(),
                                                content: None,
                                            };
                                            if let Ok(json) = serde_json::to_string(&done_msg) {
                                                let _ = browser_ws.send_with_str(json);
                                            }
                                        }
                                    }
                                }
                            } else if let Ok(worker::WebsocketEvent::Close(_)) = openai_event {
                                let _ = browser_ws.close(None, Some(""));
                                break;
                            }
                        }
                        else => break,
                    }
                }
            });

            return Response::from_websocket(client);
        }

        Response::error("Expected WebSocket", 400)
    }
}
