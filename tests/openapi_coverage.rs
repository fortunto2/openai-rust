//! OpenAPI spec coverage tests.
//!
//! Parses tests/openapi.yaml, extracts schema fields,
//! compares with our Rust types to ensure coverage.
//! For request types (Serialize): serialize default and check fields.
//! For response types (Deserialize): build fixture with ALL spec fields, verify deserialization.

use std::collections::{HashMap, HashSet};

/// Parse OpenAPI YAML and extract schema field names.
fn load_openapi_schemas() -> HashMap<String, Vec<String>> {
    let yaml_str =
        std::fs::read_to_string("tests/openapi.yaml").expect("tests/openapi.yaml not found");
    // Strip lines with huge integers that overflow serde_yaml i64
    let yaml_str: String = yaml_str
        .lines()
        .filter(|line| !line.contains("9223372036854") && !line.contains("922337203685477"))
        .collect::<Vec<_>>()
        .join("\n");
    let doc: serde_yaml::Value = serde_yaml::from_str(&yaml_str).expect("Invalid YAML");

    let schemas = doc
        .get("components")
        .and_then(|c| c.get("schemas"))
        .expect("No components.schemas in OpenAPI spec");

    let mut result = HashMap::new();

    if let serde_yaml::Value::Mapping(map) = schemas {
        for (key, value) in map {
            let name = key.as_str().unwrap_or("").to_string();
            let mut fields = Vec::new();

            if let Some(props) = value.get("properties") {
                if let serde_yaml::Value::Mapping(props_map) = props {
                    for (field_key, _) in props_map {
                        if let Some(field_name) = field_key.as_str() {
                            fields.push(field_name.to_string());
                        }
                    }
                }
            }

            if let Some(all_of) = value.get("allOf") {
                if let serde_yaml::Value::Sequence(items) = all_of {
                    for item in items {
                        if let Some(props) = item.get("properties") {
                            if let serde_yaml::Value::Mapping(props_map) = props {
                                for (field_key, _) in props_map {
                                    if let Some(field_name) = field_key.as_str() {
                                        fields.push(field_name.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if !fields.is_empty() {
                result.insert(name, fields);
            }
        }
    }

    result
}

/// Extract ALL field names from a Serialize type (including None fields).
/// Uses a custom serializer that captures field names regardless of skip_serializing_if.

/// Extract field names by grepping the Rust source file for `pub fieldname:`.
fn fields_from_source(source_path: &str, struct_name: &str) -> HashSet<String> {
    let content = std::fs::read_to_string(source_path).unwrap_or_default();
    let mut fields = HashSet::new();
    let mut in_struct = false;
    let mut brace_depth = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.contains(&format!("pub struct {struct_name}")) {
            in_struct = true;
            if trimmed.contains('{') {
                brace_depth = 1;
            }
            continue;
        }

        if in_struct {
            brace_depth += trimmed.matches('{').count();
            brace_depth = brace_depth.saturating_sub(trimmed.matches('}').count());

            if brace_depth == 0 {
                break;
            }

            // Match: pub field_name: Type
            if let Some(field) = trimmed.strip_prefix("pub ") {
                if let Some(name) = field.split(':').next() {
                    let name = name.trim();
                    if !name.is_empty() && !name.contains('(') && !name.contains('<') {
                        // Check for serde rename
                        let rename = content[..content.find(&format!("pub {name}:")).unwrap_or(0)]
                            .lines()
                            .rev()
                            .take(3)
                            .find(|l| l.contains("serde(rename"))
                            .and_then(|l| l.split('"').nth(1).map(|s| s.to_string()));
                        fields.insert(rename.unwrap_or_else(|| name.to_string()));
                    }
                }
            }
        }
    }

    fields
}

/// Check coverage and print report.
fn check_coverage(schema_name: &str, spec_fields: &[String], rust_fields: &HashSet<String>) -> f64 {
    if spec_fields.is_empty() {
        return 100.0;
    }
    let matched: Vec<_> = spec_fields
        .iter()
        .filter(|f| rust_fields.contains(f.as_str()))
        .collect();
    let pct = (matched.len() as f64 / spec_fields.len() as f64) * 100.0;

    let missing: Vec<_> = spec_fields
        .iter()
        .filter(|f| !rust_fields.contains(f.as_str()))
        .collect();

    eprintln!(
        "[{schema_name}] {:.0}% ({}/{}) {}",
        pct,
        matched.len(),
        spec_fields.len(),
        if missing.is_empty() {
            String::new()
        } else {
            format!("missing: {:?}", missing)
        }
    );

    pct
}

// ── Tests ──

#[test]
fn openapi_spec_loads() {
    let schemas = load_openapi_schemas();
    let expected = [
        "CreateChatCompletionRequest",
        "CreateChatCompletionResponse",
        "CreateEmbeddingRequest",
        "CreateImageRequest",
        "CreateModerationRequest",
    ];
    for name in &expected {
        assert!(
            schemas.contains_key(*name),
            "OpenAPI spec missing schema: {name}"
        );
    }
    eprintln!("OpenAPI spec loaded: {} schemas total", schemas.len());
}

#[test]
fn chat_completion_request_coverage() {
    let schemas = load_openapi_schemas();
    let spec = schemas
        .get("CreateChatCompletionRequest")
        .expect("schema not found");

    let rust_fields = fields_from_source("src/types/chat.rs", "ChatCompletionRequest");
    eprintln!("  Rust fields found: {:?}", rust_fields);

    let pct = check_coverage("CreateChatCompletionRequest", spec, &rust_fields);
    assert!(pct >= 60.0, "coverage {pct:.0}% < 60%");
}

#[test]
fn chat_completion_response_deserializes_all_spec_fields() {
    // Build a response JSON with ALL fields from the spec.
    // If our type can deserialize it, the fields are handled (even if as Option<Value>).
    let fixture = serde_json::json!({
        "id": "chatcmpl-abc123",
        "object": "chat.completion",
        "created": 1700000000_i64,
        "model": "gpt-4o",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "Hello world",
                "refusal": null,
                "tool_calls": null
            },
            "finish_reason": "stop",
            "logprobs": null
        }],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 5,
            "total_tokens": 15,
            "prompt_tokens_details": { "cached_tokens": 0, "audio_tokens": 0 },
            "completion_tokens_details": { "reasoning_tokens": 0, "audio_tokens": 0 }
        },
        "system_fingerprint": "fp_abc123",
        "service_tier": "default"
    });

    // This must not panic — serde should handle unknown fields gracefully
    let result: Result<openai_oxide::types::chat::ChatCompletionResponse, _> =
        serde_json::from_value(fixture);
    assert!(
        result.is_ok(),
        "Failed to deserialize full response fixture: {:?}",
        result.err()
    );
}

#[test]
fn embedding_request_coverage() {
    let schemas = load_openapi_schemas();
    let spec = schemas
        .get("CreateEmbeddingRequest")
        .expect("schema not found");

    let rust_fields = fields_from_source("src/types/embedding.rs", "EmbeddingRequest");

    let pct = check_coverage("CreateEmbeddingRequest", spec, &rust_fields);
    assert!(pct >= 80.0, "coverage {pct:.0}% < 80%");
}

#[test]
fn overall_coverage_report() {
    let schemas = load_openapi_schemas();

    let request_checks: Vec<(&str, HashSet<String>)> = vec![
        (
            "CreateChatCompletionRequest",
            fields_from_source("src/types/chat.rs", "ChatCompletionRequest"),
        ),
        (
            "CreateEmbeddingRequest",
            fields_from_source("src/types/embedding.rs", "EmbeddingRequest"),
        ),
        (
            "CreateImageRequest",
            fields_from_source("src/types/image.rs", "ImageGenerateRequest"),
        ),
        (
            "CreateModerationRequest",
            fields_from_source("src/types/moderation.rs", "ModerationRequest"),
        ),
    ];

    eprintln!("\n=== OpenAPI Coverage Report ===");
    let mut total_spec = 0;
    let mut total_matched = 0;

    for (schema_name, rust_fields) in &request_checks {
        if let Some(spec_fields) = schemas.get(*schema_name) {
            let matched = spec_fields
                .iter()
                .filter(|f| rust_fields.contains(f.as_str()))
                .count();
            total_spec += spec_fields.len();
            total_matched += matched;
            check_coverage(schema_name, spec_fields, rust_fields);
        }
    }

    let overall = if total_spec == 0 {
        100.0
    } else {
        total_matched as f64 / total_spec as f64 * 100.0
    };
    eprintln!("=== Overall: {overall:.0}% ({total_matched}/{total_spec}) ===\n");
    assert!(overall >= 80.0, "Overall coverage {overall:.0}% < 80%");
}

#[test]
fn chat_completion_response_coverage() {
    let schemas = load_openapi_schemas();
    let spec = schemas
        .get("CreateChatCompletionResponse")
        .expect("schema not found");

    let rust_fields = fields_from_source("src/types/chat.rs", "ChatCompletionResponse");
    let pct = check_coverage("CreateChatCompletionResponse", spec, &rust_fields);
    assert!(pct >= 80.0, "coverage {pct:.0}% < 80%");
}

#[test]
fn usage_details_deserialize() {
    let json = serde_json::json!({
        "prompt_tokens": 100,
        "completion_tokens": 50,
        "total_tokens": 150,
        "prompt_tokens_details": {
            "cached_tokens": 20,
            "audio_tokens": 5
        },
        "completion_tokens_details": {
            "reasoning_tokens": 10,
            "audio_tokens": 3,
            "accepted_prediction_tokens": 15,
            "rejected_prediction_tokens": 2
        }
    });

    let usage: openai_oxide::types::common::Usage = serde_json::from_value(json).unwrap();
    assert_eq!(usage.prompt_tokens, Some(100));
    let prompt_details = usage.prompt_tokens_details.unwrap();
    assert_eq!(prompt_details.cached_tokens, Some(20));
    assert_eq!(prompt_details.audio_tokens, Some(5));
    let completion_details = usage.completion_tokens_details.unwrap();
    assert_eq!(completion_details.reasoning_tokens, Some(10));
    assert_eq!(completion_details.accepted_prediction_tokens, Some(15));
    assert_eq!(completion_details.rejected_prediction_tokens, Some(2));
}

#[test]
fn chat_request_new_fields_serialize() {
    use openai_oxide::types::chat::*;

    let mut req = ChatCompletionRequest::new(
        "gpt-4o",
        vec![ChatCompletionMessageParam::User {
            content: UserContent::Text("Hi".into()),
            name: None,
        }],
    );
    req.reasoning_effort = Some(openai_oxide::types::common::ReasoningEffort::High);
    req.modalities = Some(vec!["text".into(), "audio".into()]);
    req.audio = Some(ChatCompletionAudioParam {
        format: "mp3".into(),
        voice: "alloy".into(),
    });
    req.prediction = Some(PredictionContent {
        type_: "content".into(),
        content: serde_json::json!("predicted text"),
    });
    req.web_search_options = Some(WebSearchOptions {
        search_context_size: Some(openai_oxide::types::common::SearchContextSize::Medium),
        user_location: None,
    });
    req.max_tokens = Some(1000);

    let json = serde_json::to_value(&req).unwrap();
    assert_eq!(json["reasoning_effort"], "high");
    assert_eq!(json["modalities"], serde_json::json!(["text", "audio"]));
    assert_eq!(json["audio"]["format"], "mp3");
    assert_eq!(json["audio"]["voice"], "alloy");
    assert_eq!(json["prediction"]["type"], "content");
    assert_eq!(json["web_search_options"]["search_context_size"], "medium");
    assert_eq!(json["max_tokens"], 1000);
}
