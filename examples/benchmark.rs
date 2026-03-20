//! Statistical benchmark: openai-oxide vs Python openai SDK vs async-openai.
//!
//! Runs N iterations of each test, reports median/p95/min/max.
//! Designed for publishing as competitive benchmark.
//!
//! ```bash
//! OPENAI_API_KEY=sk-... cargo run --example benchmark --features responses --release
//! ```

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

    println!("\n{ITERATIONS} iterations per test. All times include full HTTP round-trip.");
    println!("Client: openai-oxide with reqwest 0.13, gzip, HTTP/2, tcp_nodelay.");
    Ok(())
}
