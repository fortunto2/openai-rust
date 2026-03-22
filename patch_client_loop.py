import sys

with open('src/client.rs', 'r') as f:
    content = f.read()

target = '''        // Fast path: first attempt — no clone, no loop
        let mut req = self.request(method.clone(), path);
        if let Some(ref val) = body_value {
            req = req.json(val);
        }

        let response = match req.send().await {
            Ok(resp) => resp,
            Err(e) if self.config.max_retries() == 0 => return Err(OpenAIError::RequestError(e)),
            Err(e) => {
                // Enter retry path
                return self.retry_loop(method, path, &body_value, e, 1).await;
            }
        };

        Self::handle_response(response).await'''

replacement = '''        // Fast path: first attempt — no clone, no loop
        let mut req = self.request(method.clone(), path);
        if let Some(ref val) = body_value {
            req = req.json(val);
        }
        
        let mut req_built = req.build().map_err(OpenAIError::RequestError)?;
        
        for mw in &self.middlewares {
            mw.on_request(&mut req_built).await?;
        }

        let response = match self.http.execute(req_built).await {
            Ok(resp) => resp,
            Err(e) if self.config.max_retries() == 0 => return Err(OpenAIError::RequestError(e)),
            Err(e) => {
                // Enter retry path
                return self.retry_loop(method, path, &body_value, e, 1).await;
            }
        };
        
        for mw in &self.middlewares {
            mw.on_response(&response).await?;
        }

        Self::handle_response(response).await'''

content = content.replace(target, replacement)

target_retry = '''        for attempt in start_attempt..=max_retries {
            let mut req = self.request(method.clone(), path);
            if let Some(val) = body_value {
                req = req.json(val);
            }

            let response = match req.send().await {
                Ok(resp) => resp,
                Err(e) => {
                    last_error = OpenAIError::RequestError(e);
                    crate::runtime::sleep(crate::runtime::backoff_ms(attempt)).await;
                    continue;
                }
            };

            let status = response.status();'''

replacement_retry = '''        for attempt in start_attempt..=max_retries {
            let mut req = self.request(method.clone(), path);
            if let Some(val) = body_value {
                req = req.json(val);
            }
            
            let mut req_built = req.build().map_err(OpenAIError::RequestError)?;
            for mw in &self.middlewares {
                mw.on_request(&mut req_built).await?;
            }

            let response = match self.http.execute(req_built).await {
                Ok(resp) => {
                    for mw in &self.middlewares {
                        mw.on_response(&resp).await?;
                    }
                    resp
                },
                Err(e) => {
                    last_error = OpenAIError::RequestError(e);
                    crate::runtime::sleep(crate::runtime::backoff_ms(attempt)).await;
                    continue;
                }
            };

            let status = response.status();'''

content = content.replace(target_retry, replacement_retry)

with open('src/client.rs', 'w') as f:
    f.write(content)
