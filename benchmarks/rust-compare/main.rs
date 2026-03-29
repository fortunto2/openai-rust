//! Rust ecosystem benchmark: openai-oxide vs async-openai vs genai
//!
//! All three SDKs hit the same OpenAI API endpoints with identical prompts.
//! Measures end-to-end latency including HTTP, TLS, serialization, deserialization.
//!
//! ```bash
//! OPENAI_API_KEY=sk-... cargo run --release
//! ```

use std::time::Instant;

const MODEL: &str = "gpt-5.4";
const ITERATIONS: usize = 5;

fn stats(times: &mut Vec<u128>) -> (u128, u128, u128, u128) {
    times.sort();
    let min = times[0];
    let max = *times.last().unwrap();
    let median = times[times.len() / 2];
    let p95 = times[((times.len() as f64 * 0.95) as usize).min(times.len() - 1)];
    (median, p95, min, max)
}

fn print_row(test: &str, oxide: u128, ao: Option<u128>, genai: Option<u128>) {
    let ao_str = ao.map_or("N/A".to_string(), |v| format!("{v}ms"));
    let genai_str = genai.map_or("N/A".to_string(), |v| format!("{v}ms"));
    println!(
        "  {:<25} {:>8}ms {:>10} {:>10}",
        test, oxide, ao_str, genai_str
    );
}

// ── openai-oxide (Responses API) ──

async fn oxide_plain(client: &openai_oxide::OpenAI) -> u128 {
    use openai_oxide::types::responses::*;
    let req = ResponseCreateRequest::new(MODEL)
        .input("What is 2+2? One number.")
        .max_output_tokens(16);
    let t = Instant::now();
    let _ = client.responses().create(req).await.unwrap();
    t.elapsed().as_millis()
}

async fn oxide_structured(client: &openai_oxide::OpenAI) -> u128 {
    use openai_oxide::types::responses::*;
    let req = ResponseCreateRequest::new(MODEL)
        .input("List 3 programming languages with year created")
        .text(ResponseTextConfig {
            format: Some(ResponseTextFormat::JsonSchema {
                name: "langs".into(),
                description: None,
                schema: Some(lang_schema()),
                strict: Some(true),
            }),
            verbosity: None,
        })
        .max_output_tokens(200);
    let t = Instant::now();
    let _ = client.responses().create(req).await.unwrap();
    t.elapsed().as_millis()
}

async fn oxide_fc(client: &openai_oxide::OpenAI) -> u128 {
    use openai_oxide::types::responses::*;
    let req = ResponseCreateRequest::new(MODEL)
        .input("What's the weather in Tokyo?")
        .tools(vec![ResponseTool::Function {
            name: "get_weather".into(),
            description: Some("Get weather".into()),
            parameters: Some(weather_schema()),
            strict: None,
        }]);
    let t = Instant::now();
    let _ = client.responses().create(req).await.unwrap();
    t.elapsed().as_millis()
}

async fn oxide_multi(client: &openai_oxide::OpenAI) -> u128 {
    use openai_oxide::types::responses::*;
    let t = Instant::now();
    let r1 = client
        .responses()
        .create(
            ResponseCreateRequest::new(MODEL)
                .input("Remember: answer is 42.")
                .store(true)
                .max_output_tokens(32),
        )
        .await
        .unwrap();
    let _ = client
        .responses()
        .create(
            ResponseCreateRequest::new(MODEL)
                .input("What is the answer?")
                .previous_response_id(&r1.id)
                .max_output_tokens(16),
        )
        .await
        .unwrap();
    t.elapsed().as_millis()
}

async fn oxide_stream(client: &openai_oxide::OpenAI) -> u128 {
    use futures_util::StreamExt;
    use openai_oxide::types::responses::*;
    let req = ResponseCreateRequest::new(MODEL)
        .input("Explain quicksort briefly.")
        .max_output_tokens(100);
    let t = Instant::now();
    let mut stream = client.responses().create_stream(req).await.unwrap();
    while let Some(ev) = stream.next().await {
        if matches!(
            ev.unwrap(),
            ResponseStreamEvent::ResponseOutputTextDelta(_)
        ) {
            let ttft = t.elapsed().as_millis();
            while stream.next().await.is_some() {}
            return ttft;
        }
    }
    t.elapsed().as_millis()
}

async fn oxide_parallel(client: &openai_oxide::OpenAI) -> u128 {
    use openai_oxide::types::responses::*;
    let t = Instant::now();
    let (c1, c2, c3) = (client.clone(), client.clone(), client.clone());
    let (r1, r2, r3) = tokio::join!(
        async {
            c1.responses()
                .create(
                    ResponseCreateRequest::new(MODEL)
                        .input("Capital of France?")
                        .max_output_tokens(16),
                )
                .await
        },
        async {
            c2.responses()
                .create(
                    ResponseCreateRequest::new(MODEL)
                        .input("Capital of Japan?")
                        .max_output_tokens(16),
                )
                .await
        },
        async {
            c3.responses()
                .create(
                    ResponseCreateRequest::new(MODEL)
                        .input("Capital of Brazil?")
                        .max_output_tokens(16),
                )
                .await
        },
    );
    r1.unwrap();
    r2.unwrap();
    r3.unwrap();
    t.elapsed().as_millis()
}

// ── async-openai (Responses API) ──

async fn ao_plain(client: &async_openai::Client<async_openai::config::OpenAIConfig>) -> u128 {
    use async_openai::types::responses::*;
    let req = CreateResponseArgs::default()
        .model(MODEL)
        .input(InputParam::Text("What is 2+2? One number.".into()))
        .max_output_tokens(16u32)
        .build()
        .unwrap();
    let t = Instant::now();
    let _ = client.responses().create(req).await.unwrap();
    t.elapsed().as_millis()
}

async fn ao_fc(client: &async_openai::Client<async_openai::config::OpenAIConfig>) -> u128 {
    use async_openai::types::responses::*;
    let req = CreateResponseArgs::default()
        .model(MODEL)
        .input(InputParam::Text("What's the weather in Tokyo?".into()))
        .tools(vec![Tool::Function(FunctionTool {
            name: "get_weather".into(),
            description: Some("Get weather".into()),
            parameters: Some(weather_schema()),
            strict: None,
            defer_loading: None,
        })])
        .build()
        .unwrap();
    let t = Instant::now();
    let _ = client.responses().create(req).await.unwrap();
    t.elapsed().as_millis()
}

async fn ao_multi(client: &async_openai::Client<async_openai::config::OpenAIConfig>) -> u128 {
    use async_openai::types::responses::*;
    let t = Instant::now();
    let r1 = client
        .responses()
        .create(
            CreateResponseArgs::default()
                .model(MODEL)
                .input(InputParam::Text("Remember: answer is 42.".into()))
                .store(true)
                .max_output_tokens(32u32)
                .build()
                .unwrap(),
        )
        .await
        .unwrap();
    let _ = client
        .responses()
        .create(
            CreateResponseArgs::default()
                .model(MODEL)
                .input(InputParam::Text("What is the answer?".into()))
                .previous_response_id(&r1.id)
                .max_output_tokens(16u32)
                .build()
                .unwrap(),
        )
        .await
        .unwrap();
    t.elapsed().as_millis()
}

// ── genai (Chat API — multi-provider adapter) ──

async fn genai_plain(client: &genai::Client) -> u128 {
    use genai::chat::ChatRequest;
    let chat_req = ChatRequest::from_user("What is 2+2? One number.");
    let t = Instant::now();
    let _ = client.exec_chat(MODEL, chat_req, None).await.unwrap();
    t.elapsed().as_millis()
}

// ── helpers ──

fn lang_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "languages": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "name": {"type": "string"},
                        "year": {"type": "integer"}
                    },
                    "required": ["name", "year"],
                    "additionalProperties": false
                }
            }
        },
        "required": ["languages"],
        "additionalProperties": false
    })
}

fn weather_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "city": {"type": "string"},
            "unit": {"type": "string", "enum": ["celsius", "fahrenheit"]}
        },
        "required": ["city", "unit"],
        "additionalProperties": false
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Rust SDK Benchmark ({ITERATIONS} iter, model: {MODEL}) ===\n");

    // Init clients
    let oxide = openai_oxide::OpenAI::from_env()?;
    let ao = async_openai::Client::new();
    let genai_client = genai::Client::default();

    // Warmup
    println!("Warming up all three SDKs...");
    oxide_plain(&oxide).await;
    ao_plain(&ao).await;
    genai_plain(&genai_client).await;
    println!("Ready.\n");

    println!(
        "  {:<25} {:>10} {:>10} {:>10}",
        "Test", "oxide", "async-oai", "genai"
    );
    println!("  {}", "-".repeat(60));

    // Plain text
    let mut ox = Vec::new();
    let mut ao_t = Vec::new();
    let mut ge = Vec::new();
    for _ in 0..ITERATIONS {
        ox.push(oxide_plain(&oxide).await);
        ao_t.push(ao_plain(&ao).await);
        ge.push(genai_plain(&genai_client).await);
    }
    print_row(
        "Plain text",
        stats(&mut ox).0,
        Some(stats(&mut ao_t).0),
        Some(stats(&mut ge).0),
    );

    // Structured output (oxide only — Responses API with json_schema)
    let mut ox = Vec::new();
    for _ in 0..ITERATIONS {
        ox.push(oxide_structured(&oxide).await);
    }
    print_row("Structured output", stats(&mut ox).0, None, None);

    // Function calling
    let mut ox = Vec::new();
    let mut ao_t = Vec::new();
    for _ in 0..ITERATIONS {
        ox.push(oxide_fc(&oxide).await);
        ao_t.push(ao_fc(&ao).await);
    }
    print_row(
        "Function calling",
        stats(&mut ox).0,
        Some(stats(&mut ao_t).0),
        None,
    );

    // Multi-turn
    let mut ox = Vec::new();
    let mut ao_t = Vec::new();
    for _ in 0..ITERATIONS {
        ox.push(oxide_multi(&oxide).await);
        ao_t.push(ao_multi(&ao).await);
    }
    print_row(
        "Multi-turn (2 reqs)",
        stats(&mut ox).0,
        Some(stats(&mut ao_t).0),
        None,
    );

    // Streaming TTFT (oxide only — SSE stream)
    let mut ox = Vec::new();
    for _ in 0..ITERATIONS {
        ox.push(oxide_stream(&oxide).await);
    }
    print_row("Streaming TTFT", stats(&mut ox).0, None, None);

    // Parallel 3x (oxide only — tokio::join!)
    let mut ox = Vec::new();
    for _ in 0..ITERATIONS {
        ox.push(oxide_parallel(&oxide).await);
    }
    print_row("Parallel 3x", stats(&mut ox).0, None, None);

    println!("\n  {ITERATIONS} iterations, median. Model: {MODEL}");
    println!("  oxide: openai-oxide (Responses API)");
    println!("  async-oai: async-openai 0.34 (Responses API)");
    println!("  genai: genai 0.6.0-beta.12 (Chat API)");

    Ok(())
}
