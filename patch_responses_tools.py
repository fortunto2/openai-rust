import sys

with open('src/types/responses.rs', 'r') as f:
    content = f.read()

impl_response_tool = '''
impl ResponseTool {
    /// Create a standard function tool.
    pub fn function(name: impl Into<String>, description: impl Into<String>, parameters: serde_json::Value) -> Self {
        Self::Function {
            function: ResponseFunctionDef {
                name: name.into(),
                description: Some(description.into()),
                parameters: Some(parameters),
                strict: Some(true),
            }
        }
    }
    
    pub fn web_search() -> Self {
        Self::WebSearch {
            web_search: None,
        }
    }
    
    pub fn file_search() -> Self {
        Self::FileSearch {
            file_search: None,
        }
    }
    
    pub fn code_interpreter() -> Self {
        Self::CodeInterpreter {
            code_interpreter: None,
        }
    }
}
'''
if "impl ResponseTool {" not in content:
    content = content.replace("#[derive(Debug, Clone, Serialize, Deserialize)]\n#[serde(tag = \"type\")]\npub enum ResponseTool {", impl_response_tool + "\n#[derive(Debug, Clone, Serialize, Deserialize)]\n#[serde(tag = \"type\")]\npub enum ResponseTool {")

with open('src/types/responses.rs', 'w') as f:
    f.write(content)
