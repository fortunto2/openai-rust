//! Responses API example — web search + function tools.
//!
//! Run with: `OPENAI_API_KEY=sk-... cargo run --example responses_api`

use openai_oxide::OpenAI;
use openai_oxide::types::responses::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OpenAI::from_env()?;

    // Simple text input
    let request = ResponseCreateRequest::new("gpt-4o")
        .input("What are the latest developments in Rust programming?")
        .instructions("Be concise and cite sources when possible.")
        .tools(vec![ResponseTool::WebSearch {
            search_context_size: Some("medium".into()),
            user_location: None,
        }])
        .temperature(0.7)
        .max_output_tokens(1024)
        .store(true);

    let response = client.responses().create(request).await?;

    println!("Response ID: {}", response.id);
    println!("Status: {:?}", response.status);
    println!("\n{}", response.output_text());

    if let Some(usage) = &response.usage {
        println!(
            "\nTokens: {} in + {} out = {} total",
            usage.input_tokens.unwrap_or(0),
            usage.output_tokens.unwrap_or(0),
            usage.total_tokens.unwrap_or(0),
        );
    }

    // Multi-turn with previous_response_id
    let follow_up = ResponseCreateRequest::new("gpt-4o")
        .input("Can you elaborate on the async ecosystem?")
        .previous_response_id(&response.id);

    let response2 = client.responses().create(follow_up).await?;
    println!("\n--- Follow-up ---\n{}", response2.output_text());

    Ok(())
}
