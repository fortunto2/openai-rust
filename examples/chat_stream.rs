//! Streaming chat completion example.
//!
//! Run with: `OPENAI_API_KEY=sk-... cargo run --example chat_stream`

use futures_util::StreamExt;
use openai_oxide::OpenAI;
use openai_oxide::types::chat::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OpenAI::from_env()?;

    let request = ChatCompletionRequest::new(
        "gpt-4o-mini",
        vec![
            ChatCompletionMessageParam::System {
                content: "You are a helpful assistant.".into(),
                name: None,
            },
            ChatCompletionMessageParam::User {
                content: UserContent::Text("Write a haiku about Rust programming.".into()),
                name: None,
            },
        ],
    );

    let mut stream = client.chat().completions().create_stream(request).await?;

    while let Some(result) = stream.next().await {
        match result {
            Ok(chunk) => {
                for choice in &chunk.choices {
                    if let Some(content) = &choice.delta.content {
                        print!("{content}");
                    }
                    if choice.finish_reason.is_some() {
                        println!();
                    }
                }
            }
            Err(e) => {
                eprintln!("\nStream error: {e}");
                break;
            }
        }
    }

    Ok(())
}
