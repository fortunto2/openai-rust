import sys

with open('src/types/chat.rs', 'r') as f:
    content = f.read()

target_req = '''    /// Penalty for frequent tokens. Range: -2.0 to 2.0.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f64>,'''

replacement_req = '''    /// Constrains effort on reasoning for reasoning models (e.g. o1).
    /// Allowed values: "low", "medium", "high".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,

    /// The upper bound for generated tokens, including visible output tokens and reasoning tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<u32>,

    /// Service tier for latency-sensitive applications (e.g. "auto", "default").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<String>,

    /// Penalty for frequent tokens. Range: -2.0 to 2.0.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f64>,'''

content = content.replace(target_req, replacement_req)

target_builder = '''    pub fn max_tokens(mut self, max: u32) -> Self {
        self.max_tokens = Some(max);
        self
    }'''

replacement_builder = '''    pub fn max_tokens(mut self, max: u32) -> Self {
        self.max_tokens = Some(max);
        self
    }

    pub fn max_completion_tokens(mut self, max: u32) -> Self {
        self.max_completion_tokens = Some(max);
        self
    }

    pub fn reasoning_effort(mut self, effort: impl Into<String>) -> Self {
        self.reasoning_effort = Some(effort.into());
        self
    }

    pub fn service_tier(mut self, tier: impl Into<String>) -> Self {
        self.service_tier = Some(tier.into());
        self
    }'''

content = content.replace(target_builder, replacement_builder)

with open('src/types/chat.rs', 'w') as f:
    f.write(content)
