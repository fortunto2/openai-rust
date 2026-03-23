// Live test for all new features — requires OPENAI_API_KEY
//
// cargo run --example live_features_test --features structured

use futures_util::StreamExt;
use openai_oxide::OpenAI;
use openai_oxide::stream_helpers::ChatStreamEvent;
use openai_oxide::types::chat::{ChatCompletionMessageParam, ChatCompletionRequest, UserContent};

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct MathAnswer {
    steps: Vec<Step>,
    final_answer: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct Step {
    explanation: String,
    output: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct Sentiment {
    sentiment: String,
    confidence: f64,
}

fn msg(text: &str) -> Vec<ChatCompletionMessageParam> {
    vec![ChatCompletionMessageParam::User {
        content: UserContent::Text(text.into()),
        name: None,
    }]
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OpenAI::from_env()?;
    let mut passed = 0;
    let mut failed = 0;

    // ── 1. Structured Outputs (Chat) ──
    print!("1. Structured Outputs (Chat)... ");
    match client
        .chat()
        .completions()
        .parse::<MathAnswer>(
            ChatCompletionRequest::new("gpt-5.4-mini", msg("What is 2+2? Show steps."))
                .max_completion_tokens(200),
        )
        .await
    {
        Ok(result) => {
            if let Some(answer) = &result.parsed {
                println!("OK — answer: {}", answer.final_answer);
                passed += 1;
            } else {
                println!("FAIL — parsed is None");
                failed += 1;
            }
        }
        Err(e) => {
            println!("FAIL — {e}");
            failed += 1;
        }
    }

    // ── 2. Structured Outputs (Responses API) ──
    print!("2. Structured Outputs (Responses)... ");
    match client
        .responses()
        .parse::<Sentiment>(
            openai_oxide::types::responses::ResponseCreateRequest::new("gpt-5.4-mini")
                .input("I love this Rust library!")
                .max_output_tokens(100),
        )
        .await
    {
        Ok(result) => {
            if let Some(s) = &result.parsed {
                println!("OK — {}: {:.0}%", s.sentiment, s.confidence * 100.0);
                passed += 1;
            } else {
                println!("FAIL — parsed is None");
                failed += 1;
            }
        }
        Err(e) => {
            println!("FAIL — {e}");
            failed += 1;
        }
    }

    // ── 3. Stream helpers — get_final_completion ──
    print!("3. Stream get_final_completion... ");
    match client
        .chat()
        .completions()
        .create_stream_helper(
            ChatCompletionRequest::new("gpt-5.4-mini", msg("Say hello in 3 words"))
                .max_completion_tokens(20),
        )
        .await
    {
        Ok(stream) => match stream.get_final_completion().await {
            Ok(completion) => {
                let text = completion.choices[0]
                    .message
                    .content
                    .as_deref()
                    .unwrap_or("");
                println!("OK — \"{text}\"");
                passed += 1;
            }
            Err(e) => {
                println!("FAIL — {e}");
                failed += 1;
            }
        },
        Err(e) => {
            println!("FAIL — {e}");
            failed += 1;
        }
    }

    // ── 4. Stream helpers — typed events ──
    print!("4. Stream typed events... ");
    match client
        .chat()
        .completions()
        .create_stream_helper(
            ChatCompletionRequest::new("gpt-5.4-mini", msg("Count: 1,2,3"))
                .max_completion_tokens(20),
        )
        .await
    {
        Ok(mut stream) => {
            let mut got_delta = false;
            let mut got_done = false;
            while let Some(event) = stream.next().await {
                match event {
                    Ok(ChatStreamEvent::ContentDelta { .. }) => got_delta = true,
                    Ok(ChatStreamEvent::ContentDone { content }) => {
                        got_done = true;
                        print!("\"{content}\" ");
                    }
                    Ok(ChatStreamEvent::Done { .. }) => break,
                    Err(e) => {
                        println!("FAIL — {e}");
                        failed += 1;
                        break;
                    }
                    _ => {}
                }
            }
            if got_delta && got_done {
                println!("OK");
                passed += 1;
            } else if failed == 0 {
                println!("FAIL — delta={got_delta} done={got_done}");
                failed += 1;
            }
        }
        Err(e) => {
            println!("FAIL — {e}");
            failed += 1;
        }
    }

    // ── 5. Streaming retry (basic — just check stream works) ──
    print!("5. Streaming with retry enabled... ");
    match client
        .chat()
        .completions()
        .create_stream(
            ChatCompletionRequest::new("gpt-5.4-mini", msg("Hi")).max_completion_tokens(5),
        )
        .await
    {
        Ok(mut stream) => {
            let mut count = 0;
            while let Some(Ok(_)) = stream.next().await {
                count += 1;
            }
            println!("OK — {count} chunks");
            passed += 1;
        }
        Err(e) => {
            println!("FAIL — {e}");
            failed += 1;
        }
    }

    // ── 6. Typed ResponseStreamEvent ──
    print!("6. Typed ResponseStreamEvent... ");
    match client
        .responses()
        .create_stream(
            openai_oxide::types::responses::ResponseCreateRequest::new("gpt-5.4-mini")
                .input("Say hi")
                .max_output_tokens(32),
        )
        .await
    {
        Ok(mut stream) => {
            let mut got_text = false;
            let mut got_completed = false;
            while let Some(Ok(event)) = stream.next().await {
                use openai_oxide::types::responses::ResponseStreamEvent::*;
                match event {
                    OutputTextDelta { .. } => got_text = true,
                    ResponseCompleted { .. } => got_completed = true,
                    _ => {}
                }
            }
            if got_text && got_completed {
                println!("OK");
                passed += 1;
            } else {
                println!("FAIL — text={got_text} completed={got_completed}");
                failed += 1;
            }
        }
        Err(e) => {
            println!("FAIL — {e}");
            failed += 1;
        }
    }

    // ── 7. Request ID in errors ──
    print!("7. Request ID in ApiError... ");
    let bad_client = OpenAI::new("not-a-real-key");
    match bad_client
        .chat()
        .completions()
        .create(ChatCompletionRequest::new("gpt-5.4-mini", msg("test")).max_completion_tokens(1))
        .await
    {
        Err(openai_oxide::OpenAIError::ApiError {
            status, request_id, ..
        }) => {
            println!(
                "OK — status={status}, request_id={}",
                request_id.as_deref().unwrap_or("none")
            );
            passed += 1;
        }
        other => {
            println!("FAIL — unexpected: {other:?}");
            failed += 1;
        }
    }

    // ── Summary ──
    println!("\n═══════════════════════════════");
    println!("Results: {passed} passed, {failed} failed");
    if failed > 0 {
        std::process::exit(1);
    }
    Ok(())
}
