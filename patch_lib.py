import sys
with open('openai-oxide-node/src/lib.rs', 'r') as f:
    content = f.read()

content = content.replace(
    '''#[napi]
    pub async fn create_response(&self, request: serde_json::Value) -> Result<serde_json::Value> {''',
    '''#[napi(ts_args_type="request: Record<string, any>", ts_return_type="Promise<Record<string, any>>")]
    pub async fn create_response(&self, request: serde_json::Value) -> Result<serde_json::Value> {'''
)

content = content.replace(
    '''#[napi(ts_return_type="Promise<void>")]
    pub fn create_stream(
        &self,
        request: serde_json::Value,''',
    '''#[napi(ts_args_type="request: Record<string, any>, tsfn: (err: Error | null, event: Record<string, any> | null) => void", ts_return_type="Promise<void>")]
    pub fn create_stream(
        &self,
        request: serde_json::Value,'''
)

with open('openai-oxide-node/src/lib.rs', 'w') as f:
    f.write(content)
