import sys

with open('src/types/chat.rs', 'r') as f:
    content = f.read()

# Make sure we add file_search, web_search, etc
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
    
    /// Web search tool (used by gpt-4o-search models)
    pub fn web_search() -> Self {
        Self {
            type_: "web_search".to_string(),
            function: FunctionDef {
                name: "".to_string(),
                description: None,
                parameters: None,
                strict: None,
            }
        }
    }
    
    /// File search tool
    pub fn file_search() -> Self {
        Self {
            type_: "file_search".to_string(),
            function: FunctionDef {
                name: "".to_string(),
                description: None,
                parameters: None,
                strict: None,
            }
        }
    }
    
    /// Code interpreter tool
    pub fn code_interpreter() -> Self {
        Self {
            type_: "code_interpreter".to_string(),
            function: FunctionDef {
                name: "".to_string(),
                description: None,
                parameters: None,
                strict: None,
            }
        }
    }
}
'''
target = '''impl Tool {
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
}'''

content = content.replace(target, tool_impl)
with open('src/types/chat.rs', 'w') as f:
    f.write(content)
