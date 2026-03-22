import sys

with open('openai-oxide-node/src/lib.rs', 'r') as f:
    content = f.read()

# Add Chat completion proxy to Node!
chat_methods = '''
    #[napi(ts_args_type="request: Record<string, any>", ts_return_type="Promise<Record<string, any>>")]
    pub async fn create_chat_completion(&self, request: serde_json::Value) -> Result<serde_json::Value> {
        let res = self.inner.chat().completions().create_raw(&request).await.map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(res)
    }

    #[napi(ts_args_type="request: Record<string, any>, tsfn: (err: Error | null, event: Record<string, any> | null) => void", ts_return_type="Promise<void>")]
    pub async fn create_chat_stream(
        &self,
        request: serde_json::Value,
        tsfn: ThreadsafeFunction<serde_json::Value>,
    ) -> Result<()> {
        let mut body = request;
        
        // Ensure stream=true
        if let Some(obj) = body.as_object_mut() {
            obj.insert("stream".to_string(), serde_json::Value::Bool(true));
            // Force stream_options for usage tracking
            if !obj.contains_key("stream_options") {
                obj.insert("stream_options".to_string(), serde_json::json!({"include_usage": true}));
            }
        }

        match self.inner.post_stream_raw("/chat/completions", &body).await {
            Ok(mut stream) => {
                while let Some(item) = stream.next().await {
                    match item {
                        Ok(event) => {
                            tsfn.call(Ok(event), ThreadsafeFunctionCallMode::Blocking);
                        }
                        Err(e) => {
                            tsfn.call(Err(Error::from_reason(e.to_string())), ThreadsafeFunctionCallMode::Blocking);
                            break;
                        }
                    }
                }
                tsfn.call(Ok(serde_json::json!({"type": "done"})), ThreadsafeFunctionCallMode::Blocking);
            }
            Err(e) => {
                tsfn.call(Err(Error::from_reason(e.to_string())), ThreadsafeFunctionCallMode::Blocking);
            }
        }
        Ok(())
    }
'''

if "create_chat_completion" not in content:
    content = content.replace("    #[napi(ts_args_type=\"request: Record<string, any>\", ts_return_type=\"Promise<Record<string, any>>\")]\n    pub async fn create_response", chat_methods + "    #[napi(ts_args_type=\"request: Record<string, any>\", ts_return_type=\"Promise<Record<string, any>>\")]\n    pub async fn create_response")

with open('openai-oxide-node/src/lib.rs', 'w') as f:
    f.write(content)
