//! WebSocket Responses API example — persistent connection for multi-turn.
//!
//! Run with: `OPENAI_API_KEY=sk-... cargo run --example websocket --features websocket`

use futures_util::StreamExt;
use openai_oxide::OpenAI;
use openai_oxide::types::responses::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OpenAI::from_env()?;

    // --- Example 1: Simple send/receive ---
    println!("=== Example 1: Simple request ===\n");

    let mut session = client.ws_session().await?;

    let request = ResponseCreateRequest::new("gpt-4o-mini")
        .input("What is the capital of France?")
        .max_output_tokens(256);

    let response = session.send(request).await?;
    println!("Response: {}", response.output_text());
    if let Some(usage) = &response.usage {
        println!(
            "Tokens: {} in + {} out",
            usage.input_tokens.unwrap_or(0),
            usage.output_tokens.unwrap_or(0),
        );
    }

    // --- Example 2: Multi-turn via same session ---
    println!("\n=== Example 2: Multi-turn ===\n");

    let follow_up = ResponseCreateRequest::new("gpt-4o-mini")
        .input("What about Germany?")
        .previous_response_id(&response.id);

    let response2 = session.send(follow_up).await?;
    println!("Follow-up: {}", response2.output_text());

    // --- Example 3: Streaming events ---
    println!("\n=== Example 3: Streaming ===\n");

    let stream_request = ResponseCreateRequest::new("gpt-4o-mini")
        .input("Count from 1 to 5, one number per line.")
        .max_output_tokens(128);

    let mut stream = session.send_stream(stream_request).await?;
    while let Some(event) = stream.next().await {
        let event = event?;
        match event.type_.as_str() {
            "response.output_text.delta" => {
                if let Some(delta) = event.data["delta"].as_str() {
                    print!("{delta}");
                }
            }
            "response.completed" => {
                println!("\n\n[completed]");
            }
            _ => {
                // Other events: created, output_item.added, etc.
            }
        }
    }

    // --- Clean up ---
    session.close().await?;
    println!("\nSession closed.");

    Ok(())
}
