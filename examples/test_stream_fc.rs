use openai_oxide::OpenAI;
use openai_oxide::types::responses::*;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OpenAI::from_env()?;

    // Warmup
    let _ = client
        .responses()
        .create(
            ResponseCreateRequest::new("gpt-5.4")
                .input("ping")
                .max_output_tokens(16),
        )
        .await?;

    let tools = vec![ResponseTool::Function {
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
    }];

    // Test 1: Normal create() — waits for response.completed
    let mut normal_times = Vec::new();
    for _ in 0..5 {
        let t0 = Instant::now();
        let resp = client
            .responses()
            .create(
                ResponseCreateRequest::new("gpt-5.4")
                    .input("Weather in Tokyo?")
                    .tools(tools.clone()),
            )
            .await?;
        let ms = t0.elapsed().as_millis();
        normal_times.push(ms);
    }
    normal_times.sort();
    println!(
        "Normal FC:      median={}ms  min={}ms  max={}ms",
        normal_times[2], normal_times[0], normal_times[4]
    );

    // Test 2: create_stream_fc() — emits on arguments.done
    let mut stream_times = Vec::new();
    for _ in 0..5 {
        let t0 = Instant::now();
        let (mut rx, _id_rx) = client
            .responses()
            .create_stream_fc(
                ResponseCreateRequest::new("gpt-5.4")
                    .input("Weather in Moscow?")
                    .tools(tools.clone()),
            )
            .await?;
        // Time to FIRST function call
        if let Some(fc) = rx.recv().await {
            let ms = t0.elapsed().as_millis();
            stream_times.push(ms);
            let _ = fc; // would start tool execution here
        }
    }
    stream_times.sort();
    if !stream_times.is_empty() {
        println!(
            "Stream FC TTFC:  median={}ms  min={}ms  max={}ms",
            stream_times[2],
            stream_times[0],
            stream_times[stream_times.len() - 1]
        );
        println!(
            "Savings:         {}ms faster ({}%)",
            normal_times[2] as i64 - stream_times[2] as i64,
            ((normal_times[2] as f64 - stream_times[2] as f64) / normal_times[2] as f64 * 100.0)
                as i32
        );
    }

    Ok(())
}
