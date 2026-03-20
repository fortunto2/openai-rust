//! Tool calling example — model calls a function, we return the result.
//!
//! Run with: `OPENAI_API_KEY=sk-... cargo run --example tool_calling`

use openai_oxide::OpenAI;
use openai_oxide::types::chat::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OpenAI::from_env()?;

    let tools = vec![Tool {
        type_: "function".into(),
        function: FunctionDef {
            name: "get_weather".into(),
            description: Some("Get current weather for a city".into()),
            parameters: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "city": {"type": "string", "description": "City name"},
                    "unit": {"type": "string", "enum": ["celsius", "fahrenheit"]}
                },
                "required": ["city"]
            })),
            strict: Some(true),
        },
    }];

    // Step 1: Send message with tools
    let request = ChatCompletionRequest::new(
        "gpt-4o-mini",
        vec![ChatCompletionMessageParam::User {
            content: UserContent::Text("What's the weather in Tokyo?".into()),
            name: None,
        }],
    )
    .tools(tools.clone())
    .tool_choice(ToolChoice::Mode("auto".into()));

    let response = client.chat().completions().create(request).await?;
    let message = &response.choices[0].message;

    if let Some(tool_calls) = &message.tool_calls {
        for tc in tool_calls {
            println!(
                "Tool call: {} ({})",
                tc.function.name, tc.function.arguments
            );

            // Step 2: Simulate function result
            let result = r#"{"temperature": 22, "condition": "Sunny", "unit": "celsius"}"#;

            // Step 3: Send tool result back
            let follow_up = ChatCompletionRequest::new(
                "gpt-4o-mini",
                vec![
                    ChatCompletionMessageParam::User {
                        content: UserContent::Text("What's the weather in Tokyo?".into()),
                        name: None,
                    },
                    ChatCompletionMessageParam::Assistant {
                        content: None,
                        name: None,
                        tool_calls: Some(tool_calls.clone()),
                        refusal: None,
                    },
                    ChatCompletionMessageParam::Tool {
                        content: result.into(),
                        tool_call_id: tc.id.clone(),
                    },
                ],
            )
            .tools(tools.clone());

            let final_response = client.chat().completions().create(follow_up).await?;
            println!(
                "\nAssistant: {}",
                final_response.choices[0]
                    .message
                    .content
                    .as_deref()
                    .unwrap_or("")
            );
        }
    } else {
        println!("Assistant: {}", message.content.as_deref().unwrap_or(""));
    }

    Ok(())
}
