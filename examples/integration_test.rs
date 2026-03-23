//! Full integration test — exercises every oxide feature against real API.
//!
//! OPENAI_API_KEY=sk-... cargo run --example integration_test --features "responses,websocket" --release

use futures_util::StreamExt;
use openai_oxide::types::responses::*;
use openai_oxide::{OpenAI, hedged_request};
use std::time::{Duration, Instant};

const MODEL: &str = "gpt-5.4";

type R = Result<String, Box<dyn std::error::Error>>;

macro_rules! test {
    ($name:expr, $body:expr) => {{
        let t0 = Instant::now();
        let result: R = $body;
        let ms = t0.elapsed().as_millis();
        match result {
            Ok(msg) => println!("  [PASS] {:<30} {:>5}ms  {}", $name, ms, msg),
            Err(e) => println!("  [FAIL] {:<30} {:>5}ms  {}", $name, ms, e),
        }
    }};
}

#[tokio::main]
async fn main() {
    let client = OpenAI::from_env().expect("OPENAI_API_KEY not set");

    // Warmup
    let _ = client
        .responses()
        .create(
            ResponseCreateRequest::new(MODEL)
                .input("ping")
                .max_output_tokens(16),
        )
        .await;

    println!("=== openai-oxide integration test (model: {MODEL}) ===\n");

    // 1. Plain text
    test!(
        "Plain text",
        async {
            let r = client
                .responses()
                .create(
                    ResponseCreateRequest::new(MODEL)
                        .input("2+2=?")
                        .max_output_tokens(16),
                )
                .await?;
            let text = r.output_text();
            if text.is_empty() {
                return Err("empty output".into());
            }
            Ok(format!("\"{}\"", text.trim()))
        }
        .await
    );

    // 2. Structured output
    test!("Structured output (JSON)", async {
        let r = client
            .responses()
            .create(
                ResponseCreateRequest::new(MODEL)
                    .input("Name 2 planets")
                    .text(ResponseTextConfig {
                        format: Some(ResponseTextFormat::JsonSchema {
                            name: "planets".into(),
                            description: None,
                            schema: Some(serde_json::json!({
                                "type": "object",
                                "properties": {"planets": {"type": "array", "items": {"type": "string"}}},
                                "required": ["planets"],
                                "additionalProperties": false
                            })),
                            strict: Some(true),
                        }),
                        verbosity: None,
                    })
                    .max_output_tokens(100),
            )
            .await?;
        let v: serde_json::Value = serde_json::from_str(&r.output_text())?;
        let n = v["planets"].as_array().map(|a| a.len()).unwrap_or(0);
        Ok(format!("{n} planets"))
    }.await);

    // 3. Function calling
    test!(
        "Function calling",
        async {
            let r = client
                .responses()
                .create(
                    ResponseCreateRequest::new(MODEL)
                        .input("Weather in Paris?")
                        .tools(vec![ResponseTool::Function {
                            name: "get_weather".into(),
                            description: Some("Get weather".into()),
                            parameters: Some(serde_json::json!({
                                "type": "object",
                                "properties": {"city": {"type": "string"}},
                                "required": ["city"],
                                "additionalProperties": false
                            })),
                            strict: None,
                        }]),
                )
                .await?;
            let fcs = r.function_calls();
            if fcs.is_empty() {
                return Err("no function calls".into());
            }
            Ok(format!("{}({})", fcs[0].name, fcs[0].arguments))
        }
        .await
    );

    // 4. Multi-turn
    test!(
        "Multi-turn",
        async {
            let r1 = client
                .responses()
                .create(
                    ResponseCreateRequest::new(MODEL)
                        .input("Remember: X=42")
                        .store(true)
                        .max_output_tokens(32),
                )
                .await?;
            let r2 = client
                .responses()
                .create(
                    ResponseCreateRequest::new(MODEL)
                        .input("What is X?")
                        .previous_response_id(&r1.id)
                        .max_output_tokens(32),
                )
                .await?;
            let text = r2.output_text();
            if !text.contains("42") {
                return Err(format!("expected 42, got: {text}").into());
            }
            Ok("remembered 42".to_string())
        }
        .await
    );

    // 5. Streaming TTFT
    test!(
        "Streaming TTFT",
        async {
            let mut stream = client
                .responses()
                .create_stream(
                    ResponseCreateRequest::new(MODEL)
                        .input("Count 1 to 3")
                        .max_output_tokens(32),
                )
                .await?;
            let t0 = Instant::now();
            let mut ttft = None;
            while let Some(ev) = stream.next().await {
                let ev = ev?;
                if matches!(
                    ev,
                    openai_oxide::types::responses::ResponseStreamEvent::OutputTextDelta { .. }
                ) && ttft.is_none()
                {
                    ttft = Some(t0.elapsed().as_millis());
                }
                if matches!(
                    ev,
                    openai_oxide::types::responses::ResponseStreamEvent::ResponseCompleted { .. }
                ) {
                    break;
                }
            }
            Ok(format!("TTFT={}ms", ttft.unwrap_or(0)))
        }
        .await
    );

    // 6. Stream FC early parse
    test!(
        "Stream FC early parse",
        async {
            let mut handle = client
                .responses()
                .create_stream_fc(
                    ResponseCreateRequest::new(MODEL)
                        .input("Weather in Tokyo?")
                        .tools(vec![ResponseTool::Function {
                            name: "get_weather".into(),
                            description: Some("Get weather".into()),
                            parameters: Some(serde_json::json!({
                                "type": "object",
                                "properties": {"city": {"type": "string"}},
                                "required": ["city"],
                                "additionalProperties": false
                            })),
                            strict: None,
                        }]),
                )
                .await?;
            match handle.recv().await {
                Some(fc) => Ok(format!("{}({})", fc.name, fc.arguments)),
                None => {
                    let err = handle.error_now().unwrap_or("no FC received".into());
                    Err(err.into())
                }
            }
        }
        .await
    );

    // 7. Hedged request
    test!(
        "Hedged request (2x race)",
        async {
            let r = hedged_request(
                &client,
                ResponseCreateRequest::new(MODEL)
                    .input("1+1=?")
                    .max_output_tokens(16),
                Some(Duration::from_secs(2)),
            )
            .await?;
            Ok(format!("\"{}\"", r.output_text().trim()))
        }
        .await
    );

    // 8. Parallel fan-out
    test!(
        "Parallel 3x fan-out",
        async {
            let (c1, c2, c3) = (client.clone(), client.clone(), client.clone());
            let (r1, r2, r3) = tokio::join!(
                async {
                    c1.responses()
                        .create(
                            ResponseCreateRequest::new(MODEL)
                                .input("1+1?")
                                .max_output_tokens(16),
                        )
                        .await
                },
                async {
                    c2.responses()
                        .create(
                            ResponseCreateRequest::new(MODEL)
                                .input("2+2?")
                                .max_output_tokens(16),
                        )
                        .await
                },
                async {
                    c3.responses()
                        .create(
                            ResponseCreateRequest::new(MODEL)
                                .input("3+3?")
                                .max_output_tokens(16),
                        )
                        .await
                },
            );
            r1?;
            r2?;
            r3?;
            Ok("3 responses".to_string())
        }
        .await
    );

    // 9. WebSocket
    test!(
        "WebSocket session",
        async {
            let mut ws = client.ws_session().await?;
            let r = ws
                .send(
                    ResponseCreateRequest::new(MODEL)
                        .input("Hi!")
                        .max_output_tokens(16),
                )
                .await?;
            ws.close().await?;
            Ok(format!("\"{}\"", r.output_text().trim()))
        }
        .await
    );

    // 10. Prompt cache key
    test!(
        "Prompt cache key",
        async {
            let r = client
                .responses()
                .create(
                    ResponseCreateRequest::new(MODEL)
                        .instructions("You are an expert on Rust.")
                        .input("What is a lifetime?")
                        .prompt_cache_key("test-rust-expert")
                        .max_output_tokens(50),
                )
                .await?;
            let text = r.output_text();
            if text.is_empty() {
                return Err("empty".into());
            }
            Ok(format!("{} chars", text.len()))
        }
        .await
    );

    // 11. Web search
    test!(
        "Web search",
        async {
            let r = client
                .responses()
                .create(
                    ResponseCreateRequest::new(MODEL)
                        .input("Latest Rust version?")
                        .tools(vec![ResponseTool::WebSearch {
                            search_context_size: Some("low".into()),
                            user_location: None,
                        }])
                        .max_output_tokens(100),
                )
                .await?;
            let text = r.output_text();
            if text.is_empty() {
                return Err("empty".into());
            }
            Ok(format!("{} chars", text.len()))
        }
        .await
    );

    println!("\n=== Done ===");
}
