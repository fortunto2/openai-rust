# Migrating from openai-python

openai-oxide uses the same parameter names and resource structure as the official [openai-python](https://github.com/openai/openai-python) SDK. If you know the Python API, you already know openai-oxide.

## Key Differences

| Python | Rust (openai-oxide) |
|--------|---------------------|
| `client = OpenAI()` | `let client = OpenAI::from_env()?;` |
| `client.chat.completions.create(...)` | `client.chat().completions().create(...).await?` |
| `client.responses.create(...)` | `client.responses().create(...).await?` |
| `stream=True` parameter | Separate `create_stream()` method |
| Dict / Pydantic models | Typed request/response structs |
| `None` for optional fields | `Option<T>` with builder methods |
| Exception handling | `Result<T, OpenAIError>` |

## Pattern: Python to Rust

```python
# Python
response = client.responses.create(
    model="gpt-5.4",
    input="Hello",
    max_output_tokens=100,
    temperature=0.7,
)
```

```rust
// Rust
let response = client.responses().create(
    ResponseCreateRequest::new("gpt-5.4")
        .input("Hello")
        .max_output_tokens(100)
        .temperature(0.7)
).await?;
```

See the [OpenAI Docs Mapping](../openai-mapping.md) for a complete endpoint cross-reference.
