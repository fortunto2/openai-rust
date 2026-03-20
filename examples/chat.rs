//! Basic chat completion example.
//!
//! Run with: `OPENAI_API_KEY=sk-... cargo run --example chat`

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
                content: UserContent::Text("What is the capital of France?".into()),
                name: None,
            },
        ],
    );

    let response = client.chat().completions().create(request).await?;

    for choice in &response.choices {
        println!(
            "[{}] {}",
            choice.finish_reason,
            choice.message.content.as_deref().unwrap_or("")
        );
    }

    if let Some(usage) = &response.usage {
        println!(
            "\nTokens: {} prompt + {} completion = {} total",
            usage.prompt_tokens.unwrap_or(0),
            usage.completion_tokens.unwrap_or(0),
            usage.total_tokens.unwrap_or(0),
        );
    }

    Ok(())
}
