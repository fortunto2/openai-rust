use futures_util::StreamExt;
use openai_oxide::types::responses::ResponseCreateRequest;
use openai_oxide::{ClientConfig, OpenAI};
use serde::Deserialize;
use worker::*;

#[derive(Deserialize)]
struct ChatRequest {
    message: String,
    #[serde(default = "default_model")]
    model: String,
}

fn default_model() -> String {
    "gpt-5.4-mini".to_string()
}

/// Resolve API key: request header > worker secret > error
fn resolve_api_key(req: &Request, ctx: &RouteContext<()>) -> std::result::Result<String, Error> {
    if let Some(key) = req.headers().get("X-OpenAI-Key").ok().flatten() {
        if !key.is_empty() {
            return Ok(key);
        }
    }
    if let Ok(secret) = ctx.secret("OPENAI_API_KEY") {
        return Ok(secret.to_string());
    }
    Err(Error::RustError(
        "No API key. Set X-OpenAI-Key header or configure OPENAI_API_KEY secret".into(),
    ))
}

fn build_client(api_key: String, ctx: &RouteContext<()>) -> OpenAI {
    let base_url = match ctx.var("AI_GATEWAY_URL") {
        Ok(url) => url.to_string(),
        Err(_) => "https://api.openai.com/v1".to_string(),
    };
    OpenAI::with_config(ClientConfig::new(api_key).base_url(base_url))
}

fn cors_headers() -> Result<Headers> {
    let headers = Headers::new();
    headers.set("Access-Control-Allow-Origin", "*")?;
    Ok(headers)
}

const PLAYGROUND_HTML: &str = include_str!("../playground.html");

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new();

    router
        .post_async("/chat", |mut req, ctx| async move {
            let api_key = resolve_api_key(&req, &ctx)?;
            let client = build_client(api_key, &ctx);
            let body: ChatRequest = req.json().await?;

            let mut request = ResponseCreateRequest::new(body.model);
            request.input = Some(body.message.as_str().into());

            let response = client
                .responses()
                .create(request)
                .await
                .map_err(|e| Error::RustError(e.to_string()))?;

            let headers = cors_headers()?;
            headers.set("Content-Type", "application/json")?;

            Ok(Response::from_json(&serde_json::json!({
                "text": response.output_text(),
                "model": response.model,
                "usage": {
                    "input_tokens": response.usage.as_ref().map(|u| u.input_tokens),
                    "output_tokens": response.usage.as_ref().map(|u| u.output_tokens),
                }
            }))?
            .with_headers(headers))
        })
        .post_async("/chat/stream", |mut req, ctx| async move {
            let api_key = resolve_api_key(&req, &ctx)?;
            let client = build_client(api_key, &ctx);
            let body: ChatRequest = req.json().await?;

            let mut request = ResponseCreateRequest::new(body.model);
            request.input = Some(body.message.as_str().into());

            let stream = client
                .responses()
                .create_stream(request)
                .await
                .map_err(|e| Error::RustError(e.to_string()))?;

            // Map oxide SSE events → raw SSE text chunks for the client
            let sse_stream = stream.map(|event| match event {
                Ok(ev) => {
                    let event_type = ev.event_type().to_string();
                    let json = serde_json::to_string(&ev).unwrap_or_default();
                    let chunk = format!("event: {}\ndata: {}\n\n", event_type, json);
                    Ok::<Vec<u8>, Error>(chunk.into_bytes())
                }
                Err(e) => {
                    let chunk = format!("event: error\ndata: {}\n\n", e);
                    Ok(chunk.into_bytes())
                }
            });

            let headers = cors_headers()?;
            headers.set("Content-Type", "text/event-stream")?;
            headers.set("Cache-Control", "no-cache")?;

            Ok(Response::from_stream(sse_stream)?.with_headers(headers))
        })
        .options("/chat", |_, _| {
            let headers = Headers::new();
            headers.set("Access-Control-Allow-Origin", "*")?;
            headers.set("Access-Control-Allow-Methods", "POST, OPTIONS")?;
            headers.set("Access-Control-Allow-Headers", "Content-Type, X-OpenAI-Key")?;
            headers.set("Access-Control-Max-Age", "86400")?;
            Ok(Response::empty()?.with_headers(headers))
        })
        .options("/chat/stream", |_, _| {
            let headers = Headers::new();
            headers.set("Access-Control-Allow-Origin", "*")?;
            headers.set("Access-Control-Allow-Methods", "POST, OPTIONS")?;
            headers.set("Access-Control-Allow-Headers", "Content-Type, X-OpenAI-Key")?;
            headers.set("Access-Control-Max-Age", "86400")?;
            Ok(Response::empty()?.with_headers(headers))
        })
        .get("/", |_, _| {
            let headers = Headers::new();
            headers.set("Content-Type", "text/html; charset=utf-8")?;
            Ok(Response::ok(PLAYGROUND_HTML)?.with_headers(headers))
        })
        .get("/health", |_, _| Response::ok("ok"))
        .run(req, env)
        .await
}
