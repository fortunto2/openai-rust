# Structured Output

Force the model to return JSON matching a specific schema. Guarantees valid, parseable output without prompt engineering tricks.

See the official [Structured Outputs guide](https://platform.openai.com/docs/guides/structured-outputs) for schema format and limitations.

## Rust — `parse::<T>()` (recommended)

Derive `JsonSchema` on your struct and call `parse::<T>()`. The SDK auto-generates the schema and deserializes the response.

Requires feature `structured`: `cargo add openai-oxide --features structured`

```rust
use openai_oxide::OpenAI;
use openai_oxide::types::chat::*;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
struct PersonInfo {
    name: String,
    age: u32,
    occupation: String,
    company: String,
}

#[tokio::main]
async fn main() -> Result<(), openai_oxide::OpenAIError> {
    let client = OpenAI::from_env()?;

    let request = ChatCompletionRequest::new(
        "gpt-5.4-mini",
        vec![ChatCompletionMessageParam::User {
            content: UserContent::Text("I'm Alice, 30, engineer at Acme Corp.".into()),
            name: None,
        }],
    );

    let result = client.chat().completions().parse::<PersonInfo>(request).await?;
    let person = result.parsed.unwrap();
    println!("{}: {} at {}", person.name, person.occupation, person.company);
    Ok(())
}
```

Works with the Responses API too:

```rust
use openai_oxide::types::responses::ResponseCreateRequest;

let result = client.responses()
    .parse::<PersonInfo>(
        ResponseCreateRequest::new("gpt-5.4-mini")
            .input("I'm Alice, 30, engineer at Acme Corp.")
    ).await?;
```

## Rust — Manual Schema

For full control, construct the schema yourself:

```rust
{{#include ../../../examples/structured_output.rs}}
```

## Node.js

```javascript
// With Zod (npm install zod zod-to-json-schema)
const { zodParse } = require("openai-oxide/zod");
const { z } = require("zod");

const PersonInfo = z.object({
  name: z.string(),
  age: z.number(),
  occupation: z.string(),
  company: z.string(),
});

const { parsed } = await zodParse(client, {
  model: "gpt-5.4-mini",
  messages: [{ role: "user", content: "I'm Alice, 30, engineer at Acme Corp." }],
}, PersonInfo);

console.log(parsed.name); // "Alice"
```

Without Zod — pass a raw JSON schema:

```javascript
const { parsed } = await client.createChatParsed(request, "PersonInfo", jsonSchema);
```

## Python (Pydantic v2)

```python
from pydantic import BaseModel
from openai_oxide import Client

class PersonInfo(BaseModel):
    name: str
    age: int
    occupation: str
    company: str

client = Client()
result = await client.create_parsed("gpt-5.4-mini", "I'm Alice, 30, engineer at Acme Corp.", PersonInfo)
print(result.name)  # "Alice" — typed Pydantic instance
```

## Next Steps

- [Function Calling](./function-calling.md) — Combine structured output with tool use
- [Streaming](./streaming.md) — Stream with typed events
- [Responses API](./responses-api.md) — Full parameter reference
