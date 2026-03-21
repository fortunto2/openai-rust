#![allow(non_snake_case)]

use dioxus::prelude::*;
use futures_util::{SinkExt, StreamExt};
use gloo_net::websocket::{futures::WebSocket, Message};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, PartialEq)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct WsMessage {
    action: String,
    content: Option<String>,
}

fn main() {
    dioxus_logger::init(tracing::Level::INFO).expect("failed to init logger");
    launch(App);
}

pub fn App() -> Element {
    let mut messages = use_signal(Vec::<ChatMessage>::new);
    let mut input_text = use_signal(String::new);
    let mut connected = use_signal(|| false);

    let ws_task = use_coroutine(|mut rx: UnboundedReceiver<String>| async move {
        let host = web_sys::window().unwrap().location().host().unwrap();
        let protocol = web_sys::window().unwrap().location().protocol().unwrap();
        let ws_protocol = if protocol == "https:" { "wss:" } else { "ws:" };
        let ws_url = format!("{}//{}/api/ws", ws_protocol, host);
        tracing::info!("Connecting to {}", ws_url);
        
        let ws_conn = match WebSocket::open(&ws_url) {
            Ok(conn) => conn,
            Err(e) => {
                tracing::error!("Failed to open WS: {:?}", e);
                return;
            }
        };

        connected.set(true);
        let (mut write, mut read) = ws_conn.split();

        loop {
            tokio::select! {
                Some(msg_to_send) = rx.next() => {
                    let payload = WsMessage {
                        action: "send".into(),
                        content: Some(msg_to_send),
                    };
                    if let Ok(json) = serde_json::to_string(&payload) {
                        if let Err(e) = write.send(Message::Text(json)).await {
                            tracing::error!("WS send error: {:?}", e);
                            break;
                        }
                    }
                }
                Some(ws_msg) = read.next() => {
                    if let Ok(Message::Text(text)) = ws_msg {
                        if let Ok(incoming) = serde_json::from_str::<WsMessage>(&text) {
                            if incoming.action == "chunk" {
                                if let Some(chunk) = incoming.content {
                                    let mut msgs = messages.read().clone();
                                    if let Some(last) = msgs.last_mut() {
                                        if last.role == "assistant" {
                                            last.content.push_str(&chunk);
                                        } else {
                                            msgs.push(ChatMessage { role: "assistant".into(), content: chunk });
                                        }
                                    } else {
                                        msgs.push(ChatMessage { role: "assistant".into(), content: chunk });
                                    }
                                    messages.set(msgs);
                                }
                            } else if incoming.action == "done" {
                                tracing::info!("Stream done");
                            }
                        }
                    }
                }
                else => break,
            }
        }
        
        connected.set(false);
        tracing::info!("WS Disconnected");
    });

    let mut send_message = move || {
        let text = input_text.read().clone();
        if text.is_empty() { return; }

        let mut current_msgs = messages.read().clone();
        current_msgs.push(ChatMessage { role: "user".into(), content: text.clone() });
        messages.set(current_msgs);
        
        ws_task.send(text);
        input_text.set(String::new());
    };

    let status_color = if connected() { "green" } else { "red" };
    let status_text = if connected() { "Status: Connected" } else { "Status: Disconnected" };

    rsx! {
        div {
            style: "max-width: 800px; margin: 0 auto; padding: 20px; font-family: sans-serif;",
            h1 { "OpenAI Oxide + Rust WASM + Durable Objects" }
            div {
                style: "margin-bottom: 20px; color: {status_color};",
                "{status_text}"
            }
            
            div {
                style: "height: 400px; overflow-y: auto; border: 1px solid #ccc; padding: 10px; margin-bottom: 20px;",
                for msg in messages() {
                    div {
                        style: "margin-bottom: 10px; padding: 10px; border-radius: 5px;",
                        background_color: if msg.role == "user" { "#e3f2fd" } else { "#f5f5f5" },
                        strong { "{msg.role}: " }
                        "{msg.content}"
                    }
                }
            }
            
            div {
                style: "display: flex; gap: 10px;",
                input {
                    style: "flex: 1; padding: 10px; font-size: 16px;",
                    value: "{input_text}",
                    oninput: move |e| input_text.set(e.value()),
                    onkeypress: move |e| {
                        if e.key() == dioxus::events::Key::Enter {
                            send_message();
                        }
                    },
                    placeholder: "Type your message...",
                    disabled: !connected(),
                }
                button {
                    style: "padding: 10px 20px; font-size: 16px; background-color: #007bff; color: white; border: none; border-radius: 5px; cursor: pointer;",
                    onclick: move |_| send_message(),
                    disabled: !connected(),
                    "Send"
                }
            }
        }
    }
}