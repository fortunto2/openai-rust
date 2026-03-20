//! Statistical benchmark: openai-oxide vs Python openai SDK vs async-openai.
//!
//! Runs N iterations of each test, reports median/p95/min/max.
//! Designed for publishing as competitive benchmark.
//!
//! ```bash
//! OPENAI_API_KEY=sk-... cargo run --example benchmark --features responses --release
//! ```

use futures_util::StreamExt;
use openai_oxide::OpenAI;
use openai_oxide::types::responses::*;
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OpenAI::from_env()?;

    // Warmup
    println!("Warming up (TLS + HTTP/2)...");
    client
        .responses()
        .create(ResponseCreateRequest::new(MODEL).input("ping"))
        .await?;
    println!("Ready.\n");

    println!("=== openai-oxide benchmark ({ITERATIONS} iterations, model: {MODEL}) ===\n");
    println!(
        "{:<25} {:>8} {:>8} {:>8} {:>8}",
        "Test", "Median", "P95", "Min", "Max"
    );
    println!("{}", "-".repeat(65));

    // ── Test 1: Plain text ──
    let mut times = Vec::new();
    for _ in 0..ITERATIONS {
        let req = ResponseCreateRequest::new(MODEL)
            .input("What is the capital of France? One word.")
            .max_output_tokens(16);
        let t0 = Instant::now();
        let _ = client.responses().create(req).await?;
        times.push(t0.elapsed().as_millis());
    }
    let (med, p95, min, max) = stats(&mut times);
    println!(
        "{:<25} {:>6}ms {:>6}ms {:>6}ms {:>6}ms",
        "Plain text", med, p95, min, max
    );

    // ── Test 2: Structured output ──
    let mut times = Vec::new();
    for _ in 0..ITERATIONS {
        let req = ResponseCreateRequest::new(MODEL)
            .input("List 3 programming languages with year created")
            .text(ResponseTextConfig {
                format: Some(ResponseTextFormat::JsonSchema {
                    name: "languages".into(),
                    description: None,
                    schema: Some(serde_json::json!({
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
                    })),
                    strict: Some(true),
                }),
                verbosity: None,
            })
            .max_output_tokens(200);
        let t0 = Instant::now();
        let _ = client.responses().create(req).await?;
        times.push(t0.elapsed().as_millis());
    }
    let (med, p95, min, max) = stats(&mut times);
    println!(
        "{:<25} {:>6}ms {:>6}ms {:>6}ms {:>6}ms",
        "Structured output", med, p95, min, max
    );

    // ── Test 3: Function calling ──
    let mut times = Vec::new();
    for _ in 0..ITERATIONS {
        let req = ResponseCreateRequest::new(MODEL)
            .input("What's the weather in Tokyo?")
            .tools(vec![ResponseTool::Function {
                name: "get_weather".into(),
                description: Some("Get weather".into()),
                parameters: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "city": {"type": "string"},
                        "unit": {"type": "string", "enum": ["celsius", "fahrenheit"]}
                    },
                    "required": ["city", "unit"],
                    "additionalProperties": false
                })),
                strict: None,
            }]);
        let t0 = Instant::now();
        let _ = client.responses().create(req).await?;
        times.push(t0.elapsed().as_millis());
    }
    let (med, p95, min, max) = stats(&mut times);
    println!(
        "{:<25} {:>6}ms {:>6}ms {:>6}ms {:>6}ms",
        "Function calling", med, p95, min, max
    );

    // ── Test 4: Multi-turn ──
    let mut times = Vec::new();
    for _ in 0..ITERATIONS {
        let t0 = Instant::now();
        let r1 = client
            .responses()
            .create(
                ResponseCreateRequest::new(MODEL)
                    .input("Remember: the answer is 42.")
                    .store(true)
                    .max_output_tokens(32),
            )
            .await?;
        let _ = client
            .responses()
            .create(
                ResponseCreateRequest::new(MODEL)
                    .input("What is the answer?")
                    .previous_response_id(&r1.id)
                    .max_output_tokens(16),
            )
            .await?;
        times.push(t0.elapsed().as_millis());
    }
    let (med, p95, min, max) = stats(&mut times);
    println!(
        "{:<25} {:>6}ms {:>6}ms {:>6}ms {:>6}ms",
        "Multi-turn (2 reqs)", med, p95, min, max
    );

    // ── Test 5: Web search ──
    let mut times = Vec::new();
    for _ in 0..ITERATIONS {
        let req = ResponseCreateRequest::new(MODEL)
            .input("What is the latest Rust version?")
            .tools(vec![ResponseTool::WebSearch {
                search_context_size: Some("low".into()),
                user_location: None,
            }])
            .max_output_tokens(100);
        let t0 = Instant::now();
        let _ = client.responses().create(req).await?;
        times.push(t0.elapsed().as_millis());
    }
    let (med, p95, min, max) = stats(&mut times);
    println!(
        "{:<25} {:>6}ms {:>6}ms {:>6}ms {:>6}ms",
        "Web search", med, p95, min, max
    );

    // ── Test 6: Complex structured output (nested schema) ──
    let mut times = Vec::new();
    let complex_schema = serde_json::json!({
        "type": "object",
        "properties": {
            "company": {
                "type": "object",
                "properties": {
                    "name": {"type": "string"},
                    "founded": {"type": "integer"},
                    "ceo": {"type": "string"},
                    "products": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "name": {"type": "string"},
                                "category": {"type": "string", "enum": ["hardware", "software", "service"]},
                                "revenue_billions": {"type": "number"},
                                "active": {"type": "boolean"}
                            },
                            "required": ["name", "category", "revenue_billions", "active"],
                            "additionalProperties": false
                        }
                    }
                },
                "required": ["name", "founded", "ceo", "products"],
                "additionalProperties": false
            },
            "competitors": {
                "type": "array",
                "items": {"type": "string"}
            },
            "summary": {"type": "string"}
        },
        "required": ["company", "competitors", "summary"],
        "additionalProperties": false
    });
    for _ in 0..ITERATIONS {
        let req = ResponseCreateRequest::new(MODEL)
            .input("Analyze Apple Inc: products with revenue, competitors, summary.")
            .text(ResponseTextConfig {
                format: Some(ResponseTextFormat::JsonSchema {
                    name: "company_analysis".into(),
                    description: None,
                    schema: Some(complex_schema.clone()),
                    strict: Some(true),
                }),
                verbosity: None,
            })
            .max_output_tokens(800);
        let t0 = Instant::now();
        let resp = client.responses().create(req).await?;
        // Parse to verify correctness
        let _: serde_json::Value = serde_json::from_str(&resp.output_text())?;
        times.push(t0.elapsed().as_millis());
    }
    let (med, p95, min, max) = stats(&mut times);
    println!(
        "{:<25} {:>6}ms {:>6}ms {:>6}ms {:>6}ms",
        "Nested structured", med, p95, min, max
    );

    // ── Test 7: Agent loop (3-step: FC → result → structured response) ──
    let mut times = Vec::new();
    for _ in 0..ITERATIONS {
        let t0 = Instant::now();

        // Step 1: Model calls function
        let step1 = client
            .responses()
            .create(
                ResponseCreateRequest::new(MODEL)
                    .input("What's the weather in Tokyo and what should I wear?")
                    .tools(vec![ResponseTool::Function {
                        name: "get_weather".into(),
                        description: Some("Get weather for a city".into()),
                        parameters: Some(serde_json::json!({
                            "type": "object",
                            "properties": {
                                "city": {"type": "string"},
                                "unit": {"type": "string", "enum": ["celsius", "fahrenheit"]}
                            },
                            "required": ["city", "unit"],
                            "additionalProperties": false
                        })),
                        strict: None,
                    }])
                    .store(true),
            )
            .await?;

        // Step 2: Extract real call_id, provide tool result via raw API
        let call_id = step1
            .function_calls()
            .first()
            .map(|fc| fc.call_id.clone())
            .unwrap_or_else(|| "call_1".into());
        let _step2: serde_json::Value = client
            .responses()
            .create_raw(&serde_json::json!({
                "model": MODEL,
                "previous_response_id": step1.id,
                "max_output_tokens": 200,
                "input": [{
                    "type": "function_call_output",
                    "call_id": call_id,
                    "output": "{\"temp\":22,\"condition\":\"sunny\",\"humidity\":45}"
                }],
                "text": {
                    "format": {
                        "type": "json_schema",
                        "name": "recommendation",
                        "strict": true,
                        "schema": {
                            "type": "object",
                            "properties": {
                                "outfit": {"type": "string"},
                                "accessories": {"type": "array", "items": {"type": "string"}},
                                "warning": {"type": "string"}
                            },
                            "required": ["outfit", "accessories", "warning"],
                            "additionalProperties": false
                        }
                    }
                }
            }))
            .await?;

        times.push(t0.elapsed().as_millis());
    }
    let (med, p95, min, max) = stats(&mut times);
    println!(
        "{:<25} {:>6}ms {:>6}ms {:>6}ms {:>6}ms",
        "Agent loop (2-step)", med, p95, min, max
    );

    // ── Test 8: Rapid-fire (5 sequential simple calls) ──
    let mut times = Vec::new();
    for _ in 0..ITERATIONS {
        let t0 = Instant::now();
        for i in 1..=5 {
            let _ = client
                .responses()
                .create(
                    ResponseCreateRequest::new(MODEL)
                        .input(format!("What is {i}+{i}? Reply with just the number."))
                        .max_output_tokens(16),
                )
                .await?;
        }
        times.push(t0.elapsed().as_millis());
    }
    let (med, p95, min, max) = stats(&mut times);
    println!(
        "{:<25} {:>6}ms {:>6}ms {:>6}ms {:>6}ms",
        "Rapid-fire (5 calls)", med, p95, min, max
    );

    // ── Test 9: Prompt cache (repeated system prompt with cache key) ──
    // First call: seed the cache. Subsequent calls: use cached prefix.
    let system_prompt = "You are a senior software architect with 20 years of experience in distributed systems, microservices, and cloud-native architectures. Always provide specific, actionable advice with code examples where relevant. Consider scalability, maintainability, and security in every recommendation.";
    let _ = client
        .responses()
        .create(
            ResponseCreateRequest::new(MODEL)
                .instructions(system_prompt)
                .input("ping")
                .prompt_cache_key("bench-architect")
                .max_output_tokens(16),
        )
        .await?;
    // Now measure cached calls
    let mut times = Vec::new();
    for _ in 0..ITERATIONS {
        let t0 = Instant::now();
        let _ = client
            .responses()
            .create(
                ResponseCreateRequest::new(MODEL)
                    .instructions(system_prompt)
                    .input("How should I design a rate limiter for an API gateway?")
                    .prompt_cache_key("bench-architect")
                    .prompt_cache_retention("24h")
                    .max_output_tokens(200),
            )
            .await?;
        times.push(t0.elapsed().as_millis());
    }
    let (med, p95, min, max) = stats(&mut times);
    println!(
        "{:<25} {:>6}ms {:>6}ms {:>6}ms {:>6}ms",
        "Prompt-cached", med, p95, min, max
    );

    // ── Test 10: Streaming TTFT (Time To First Token) ──
    let mut times = Vec::new();
    for _ in 0..ITERATIONS {
        let req = ResponseCreateRequest::new(MODEL)
            .input("Explain quicksort in 3 sentences.")
            .max_output_tokens(200);
        let t0 = Instant::now();
        let mut stream = client.responses().create_stream(req).await?;
        // Measure time to first content delta
        while let Some(event) = stream.next().await {
            let ev = event?;
            if ev.type_ == "response.output_text.delta" {
                times.push(t0.elapsed().as_millis());
                break;
            }
        }
        // Drain remaining events
        while let Some(_) = stream.next().await {}
    }
    if !times.is_empty() {
        let (med, p95, min, max) = stats(&mut times);
        println!(
            "{:<25} {:>6}ms {:>6}ms {:>6}ms {:>6}ms",
            "Streaming TTFT", med, p95, min, max
        );
    }

    // ── Test 11: Parallel fan-out (3 requests via HTTP/2 multiplex) ──
    // Clone client — shares same connection pool (Arc<reqwest::Client> inside)
    let mut times = Vec::new();
    for _ in 0..ITERATIONS {
        let t0 = Instant::now();
        let (c1, c2, c3) = (client.clone(), client.clone(), client.clone());
        let (r1, r2, r3) = tokio::join!(
            async {
                c1.responses()
                    .create(
                        ResponseCreateRequest::new(MODEL)
                            .input("Capital of France? One word.")
                            .max_output_tokens(16),
                    )
                    .await
            },
            async {
                c2.responses()
                    .create(
                        ResponseCreateRequest::new(MODEL)
                            .input("Capital of Japan? One word.")
                            .max_output_tokens(16),
                    )
                    .await
            },
            async {
                c3.responses()
                    .create(
                        ResponseCreateRequest::new(MODEL)
                            .input("Capital of Brazil? One word.")
                            .max_output_tokens(16),
                    )
                    .await
            },
        );
        r1?;
        r2?;
        r3?;
        times.push(t0.elapsed().as_millis());
    }
    let (med, p95, min, max) = stats(&mut times);
    println!(
        "{:<25} {:>6}ms {:>6}ms {:>6}ms {:>6}ms",
        "Parallel 3x (fan-out)", med, p95, min, max
    );

    // ── Test 12: Hedged request (send 2, take first) ──
    let mut times = Vec::new();
    for _ in 0..ITERATIONS {
        let t0 = Instant::now();
        let (c1, c2) = (client.clone(), client.clone());
        tokio::select! {
            r = async { c1.responses().create(ResponseCreateRequest::new(MODEL).input("What is 7*8? Number only.").max_output_tokens(16)).await } => { r?; }
            r = async { c2.responses().create(ResponseCreateRequest::new(MODEL).input("What is 7*8? Number only.").max_output_tokens(16)).await } => { r?; }
        }
        times.push(t0.elapsed().as_millis());
    }
    let (med, p95, min, max) = stats(&mut times);
    println!(
        "{:<25} {:>6}ms {:>6}ms {:>6}ms {:>6}ms",
        "Hedged (2x race)", med, p95, min, max
    );

    println!("\n{ITERATIONS} iterations per test. All times include full HTTP round-trip.");
    println!("Client: openai-oxide with reqwest 0.13, gzip, HTTP/2, tcp_nodelay.");
    Ok(())
}
