//! Structured output with JSON Schema — model returns a validated JSON object.
//!
//! Run with: `OPENAI_API_KEY=sk-... cargo run --example structured_output`

use openai_oxide::OpenAI;
use openai_oxide::types::chat::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OpenAI::from_env()?;

    let request = ChatCompletionRequest::new(
        "gpt-4o-mini",
        vec![
            ChatCompletionMessageParam::System {
                content: "Extract structured data from user messages.".into(),
                name: None,
            },
            ChatCompletionMessageParam::User {
                content: UserContent::Text(
                    "My name is Alice, I'm 30, and I work as a software engineer at Acme Corp."
                        .into(),
                ),
                name: None,
            },
        ],
    )
    .response_format(ResponseFormat::JsonSchema {
        json_schema: JsonSchema {
            name: "person_info".into(),
            description: Some("Extracted person information".into()),
            schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {"type": "string"},
                    "age": {"type": "integer"},
                    "occupation": {"type": "string"},
                    "company": {"type": "string"}
                },
                "required": ["name", "age", "occupation", "company"],
                "additionalProperties": false
            })),
            strict: Some(true),
        },
    });

    let response = client.chat().completions().create(request).await?;
    let content = response.choices[0]
        .message
        .content
        .as_deref()
        .unwrap_or("{}");

    let parsed: serde_json::Value = serde_json::from_str(content)?;
    println!("Extracted data:");
    println!("  Name: {}", parsed["name"]);
    println!("  Age: {}", parsed["age"]);
    println!("  Occupation: {}", parsed["occupation"]);
    println!("  Company: {}", parsed["company"]);

    Ok(())
}
