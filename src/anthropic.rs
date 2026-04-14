// Anthropic helpers.
//
// Anthropic via OpenRouter or direct API.
// Provider-specific capabilities and request decoration.
//
// Docs: <https://docs.anthropic.com/en/api/messages>

use serde_json::Value;

/// Anthropic prompt caching configuration.
///
/// ```ignore
/// let cache = anthropic::CacheControl::ephemeral("1h");
/// req.cache_control = Some(cache.to_value());
/// ```
#[derive(Debug, Clone)]
pub struct CacheControl {
    pub ttl: String,
}

impl CacheControl {
    /// Ephemeral cache with TTL (e.g. "5m", "1h").
    pub fn ephemeral(ttl: &str) -> Self {
        Self {
            ttl: ttl.to_string(),
        }
    }

    /// Default: 1 hour TTL for agent workloads.
    pub fn default_agent() -> Self {
        Self::ephemeral("1h")
    }

    /// Serialize for request body injection.
    pub fn to_value(&self) -> Value {
        serde_json::json!({"type": "ephemeral", "ttl": self.ttl})
    }
}

/// Known Anthropic model capabilities.
///
/// ```ignore
/// if anthropic::is_anthropic_model("anthropic/claude-sonnet-4.6:beta") {
///     // apply anthropic-specific behavior
/// }
/// ```
pub fn is_anthropic_model(model: &str) -> bool {
    model.starts_with("anthropic/") || model.starts_with("claude")
}

/// Anthropic models (Opus 4.6, Sonnet 4.6) reject assistant message as last in conversation.
/// Haiku works via Bedrock which allows prefill.
pub fn supports_assistant_prefill(model: &str) -> bool {
    if !is_anthropic_model(model) {
        return true;
    }
    // Haiku via Bedrock supports prefill; Opus/Sonnet via OpenRouter don't
    model.contains("haiku")
}

/// Decorate a request JSON body with Anthropic-specific fields.
///
/// Adds cache_control and provider pinning for OpenRouter.
/// Idempotent — safe to call multiple times.
pub fn decorate_request(body: &mut Value, cache: Option<&CacheControl>) {
    if let Value::Object(map) = body {
        if let Some(cache) = cache {
            map.insert("cache_control".to_string(), cache.to_value());
        }
        // Pin to Anthropic provider on OpenRouter
        if let Ok(prefs) = crate::openrouter::ProviderPreferences::pinned("Anthropic").to_value() {
            map.insert("provider".to_string(), prefs);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_anthropic_model() {
        assert!(is_anthropic_model("anthropic/claude-sonnet-4.6:beta"));
        assert!(is_anthropic_model("anthropic/claude-opus-4.6:beta"));
        assert!(is_anthropic_model("claude-3-haiku-20240307"));
        assert!(!is_anthropic_model("gpt-5.4"));
        assert!(!is_anthropic_model("nvidia/nemotron-120b"));
        assert!(!is_anthropic_model("not-claude-model"));
    }

    #[test]
    fn test_supports_prefill() {
        assert!(!supports_assistant_prefill(
            "anthropic/claude-opus-4.6:beta",
        ));
        assert!(!supports_assistant_prefill(
            "anthropic/claude-sonnet-4.6:beta",
        ));
        assert!(supports_assistant_prefill("anthropic/claude-haiku-4.5"));
        assert!(supports_assistant_prefill("gpt-5.4"));
    }

    #[test]
    fn test_cache_control() {
        let cache = CacheControl::ephemeral("1h");
        let v = cache.to_value();
        assert_eq!(v["type"], "ephemeral");
        assert_eq!(v["ttl"], "1h");
    }

    #[test]
    fn test_decorate_request() {
        let mut body = serde_json::json!({
            "model": "anthropic/claude-sonnet-4.6:beta",
            "messages": []
        });
        decorate_request(&mut body, Some(&CacheControl::default_agent()));
        assert_eq!(body["cache_control"]["ttl"], "1h");
        assert!(body["provider"]["only"].is_array());
    }
}
