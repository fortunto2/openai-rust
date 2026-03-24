# Webhook Verification

Verify OpenAI webhook signatures to ensure payloads are authentic and not replayed.

Requires feature `webhooks`: `cargo add openai-oxide --features webhooks`

See the official [Webhooks documentation](https://platform.openai.com/docs/guides/webhooks) for setup.

## Usage

```rust
use openai_oxide::resources::webhooks::Webhooks;

// Initialize with your webhook secret (from OpenAI dashboard)
let wh = Webhooks::new("whsec_YOUR_WEBHOOK_SECRET")?;

// In your HTTP handler — extract headers and body
let signature = headers.get("webhook-signature").unwrap();
let timestamp = headers.get("webhook-timestamp").unwrap();

// Verify and parse in one call
let event: serde_json::Value = wh.unwrap(body_bytes, signature, timestamp)?;

// Or verify only (without parsing)
wh.verify(body_bytes, signature, timestamp)?;
```

## Security

- HMAC-SHA256 signature validation
- Timestamp replay protection (5-minute tolerance)
- Supports multiple signature versions in header
- Base64-encoded `whsec_` secrets (auto-stripped)

## Next Steps

- [API Reference](../api-reference.md) — Full method signatures
