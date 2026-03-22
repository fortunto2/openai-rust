import sys

with open('src/types/chat.rs', 'r') as f:
    content = f.read()

# 1. Developer role
target_msg_param = '''pub enum ChatCompletionMessageParam {
    #[serde(rename = "system")]
    System {
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
    #[serde(rename = "user")]'''

replacement_msg_param = '''pub enum ChatCompletionMessageParam {
    #[serde(rename = "system")]
    System {
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
    #[serde(rename = "developer")]
    Developer {
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
    #[serde(rename = "user")]'''
content = content.replace(target_msg_param, replacement_msg_param)

# 2. Add Developer builder methods
target_role_msg = '''impl ChatCompletionMessageParam {
    /// Create a new user message
    pub fn user(content: impl Into<String>) -> Self {'''

replacement_role_msg = '''impl ChatCompletionMessageParam {
    /// Create a new developer message (used by reasoning models)
    pub fn developer(content: impl Into<String>) -> Self {
        Self::Developer {
            content: content.into(),
            name: None,
        }
    }

    /// Create a new user message
    pub fn user(content: impl Into<String>) -> Self {'''
content = content.replace(target_role_msg, replacement_role_msg)

# 3. Add Builder methods for built-in tools
tool_impl = '''
impl Tool {
    /// Create a standard function tool.
    pub fn function(name: impl Into<String>, description: impl Into<String>, parameters: serde_json::Value) -> Self {
        Self {
            type_: "function".to_string(),
            function: FunctionDef {
                name: name.into(),
                description: Some(description.into()),
                parameters: Some(parameters),
                strict: Some(true),
            }
        }
    }
}
'''
if "impl Tool {" not in content:
    content = content.replace("#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct Tool {", tool_impl + "\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct Tool {")

with open('src/types/chat.rs', 'w') as f:
    f.write(content)
