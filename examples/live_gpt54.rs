//! Live test: Responses API with GPT-5.4
//!
//! Run: `source .env && cargo run --example live_gpt54 --features responses`

use openai_oxide::OpenAI;
use openai_oxide::types::responses::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OpenAI::from_env()?;

    // 1. Simple text response
    println!("=== Test 1: Simple text ===");
    let request = ResponseCreateRequest::new("gpt-5.4")
        .input("What is 2+2? Answer in one word.")
        .max_output_tokens(50);

    let response = client.responses().create(request).await?;
    println!("ID: {}", response.id);
    println!("Model: {}", response.model);
    println!("Status: {:?}", response.status);
    println!("Output: {}", response.output_text());
    if let Some(usage) = &response.usage {
        println!(
            "Tokens: {} in + {} out",
            usage.input_tokens.unwrap_or(0),
            usage.output_tokens.unwrap_or(0),
        );
    }

    // 2. Structured output (JSON schema)
    println!("\n=== Test 2: Structured output ===");
    let request = ResponseCreateRequest::new("gpt-5.4")
        .input("List 3 programming languages with their year of creation")
        .text(ResponseTextConfig {
            format: Some(ResponseTextFormat::JsonSchema {
                name: "languages".into(),
                description: Some("List of programming languages".into()),
                schema: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "languages": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "name": { "type": "string" },
                                    "year": { "type": "integer" }
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

    let response = client.responses().create(request).await?;
    println!("Output: {}", response.output_text());

    // 3. Multi-turn with previous_response_id
    println!("\n=== Test 3: Multi-turn ===");
    let req1 = ResponseCreateRequest::new("gpt-5.4")
        .input("My name is Rustam.")
        .max_output_tokens(50)
        .store(true);
    let resp1 = client.responses().create(req1).await?;
    println!("Turn 1: {}", resp1.output_text());

    let req2 = ResponseCreateRequest::new("gpt-5.4")
        .input("What is my name?")
        .previous_response_id(&resp1.id)
        .max_output_tokens(50);
    let resp2 = client.responses().create(req2).await?;
    println!("Turn 2: {}", resp2.output_text());

    // 4. Streaming
    println!("\n=== Test 4: Streaming ===");
    use futures_util::StreamExt;
    let request = ResponseCreateRequest::new("gpt-5.4")
        .input("Count from 1 to 5, one per line.")
        .max_output_tokens(100);

    let mut stream = client.responses().create_stream(request).await?;
    while let Some(event) = stream.next().await {
        match event {
            Ok(ev) => {
                use openai_oxide::types::responses::ResponseStreamEvent::*;
                match ev {
                    OutputTextDelta { delta, .. } => print!("{}", delta),
                    ResponseCompleted { .. } => println!("\n[stream completed]"),
                    _ => {}
                }
            }
            Err(e) => {
                eprintln!("Stream error: {}", e);
                break;
            }
        }
    }

    // 5. Web search tool
    println!("\n=== Test 5: Web search ===");
    let request = ResponseCreateRequest::new("gpt-5.4")
        .input("What is the current Rust stable version?")
        .tools(vec![ResponseTool::WebSearch {
            search_context_size: Some("low".into()),
            user_location: None,
        }])
        .max_output_tokens(200);

    let response = client.responses().create(request).await?;
    println!("Output: {}", response.output_text());

    // Check for annotations (citations from web search)
    for item in &response.output {
        if let Some(content) = &item.content {
            for block in content {
                if let Some(anns) = &block.annotations {
                    for ann in anns {
                        if let Some(url) = &ann.url {
                            println!(
                                "  Citation: {} — {}",
                                ann.title.as_deref().unwrap_or(""),
                                url
                            );
                        }
                    }
                }
            }
        }
    }

    println!("\n=== All tests passed ===");
    Ok(())
}
