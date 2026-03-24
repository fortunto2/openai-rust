# Quick Start

Set your API key:

```bash
export OPENAI_API_KEY="sk-..."
```

## Rust

```rust
use openai_oxide::{OpenAI, types::responses::*};

#[tokio::main]
async fn main() -> Result<(), openai_oxide::OpenAIError> {
    let client = OpenAI::from_env()?;

    let response = client.responses().create(
        ResponseCreateRequest::new("gpt-5.4-mini")
            .input("Explain quantum computing in one sentence.")
            .max_output_tokens(100)
    ).await?;

    println!("{}", response.output_text());
    Ok(())
}
```

## Node.js

```javascript
const { Client } = require("openai-oxide");

const client = new Client();
const text = await client.createText("gpt-5.4-mini", "Hello from Node!");
console.log(text);
```

## Python

```python
import asyncio, json
from openai_oxide import Client

async def main():
    client = Client()
    res = json.loads(await client.create("gpt-5.4-mini", "Hello from Python!"))
    print(res["text"])

asyncio.run(main())
```

---

## Drop-in Migration

Switch from the official OpenAI SDK by changing **one import line**. Rest of your code stays the same.

### Python

```diff
- from openai import AsyncOpenAI
+ from openai_oxide.compat import AsyncOpenAI
```

Full working example (mirrors official `openai` examples/parsing.py):

```python
{{#include ../../openai-oxide-python/examples/parsing.py}}
```

### Node.js

```diff
- const OpenAI = require('openai');
+ const { OpenAI } = require('openai-oxide/compat');
```

Full working example (mirrors official `openai` SDK):

```javascript
{{#include ../../openai-oxide-node/examples/demo-compat.js}}
```
