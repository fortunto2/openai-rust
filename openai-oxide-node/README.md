# openai-oxide-node

Lightning-fast, memory-safe Node.js bindings for the [`openai-oxide`](https://github.com/your-org/openai-oxide) Rust client, powered by [NAPI-RS](https://napi.rs/).

This package provides a high-performance, native integration with the OpenAI API directly from Node.js, sidestepping pure-JavaScript serialization overhead and leveraging Rust's robust architecture (such as persistent WebSockets and native async tasks).

## Features

- 🚀 **Native Performance**: Written in Rust, running via NAPI. No JS JSON serialization bottlenecks.
- 🔄 **Native Streaming**: Rust pushes stream events directly into the Node.js Event Loop without blocking.
- 🔌 **Persistent WebSockets**: Keep a "hot" connection to the Responses API for multi-turn, low-latency interactions.
- 🛡 **Memory Safe**: Powered by Rust.

## Installation

You must first have Rust and Cargo installed, as this package builds the native binary during installation.

```bash
# Using pnpm (recommended)
pnpm install

# Build the native module
pnpm run build
```

## Quick Start

### 1. Basic Request

Make a simple request and receive a fully parsed native JavaScript object back. NAPI handles the Rust-to-V8 conversion natively.

```javascript
const { Client } = require('openai-oxide-node');

async function main() {
    const client = new Client();
    
    const request = {
        model: "gpt-4o-mini",
        input: "Say hello to Node.js from Rust via NAPI!",
        temperature: 0.7
    };

    const response = await client.createResponse(request);
    console.log(response.output[0].content[0].text);
}

main();
```

### 2. Streaming (with AsyncIterator)

Stream tokens as they are generated. The native module supports pushing events directly to a JS callback. You can easily wrap this in a native `AsyncIterator` for modern `for await` loops. 

See the full example in [`examples/02_streaming.js`](examples/02_streaming.js) to see how to wrap the callback in a beautiful generator.

```javascript
const { Client } = require('openai-oxide-node');

const client = new Client();

// The native method takes a callback
client.createStream({ model: "gpt-4o-mini", input: "Write a haiku" }, (err, event) => {
    if (err) return console.error(err);
    if (event.type === 'response.output_text.delta') {
        process.stdout.write(event.delta);
    }
});
```

### 3. Persistent WebSocket Sessions

For low-latency applications, you can open a persistent WebSocket connection to the Responses API. Rust holds the connection open, and you simply await the results in JS.

```javascript
const { Client } = require('openai-oxide-node');

async function main() {
    const client = new Client();
    
    console.log("Connecting WS...");
    const session = await client.wsSession();
    
    // First request
    const res1 = await session.send("gpt-4o-mini", "Say ping");
    console.log("->", res1.output[0].content[0].text);

    // Second request reuses the exact same hot connection!
    const res2 = await session.send("gpt-4o-mini", "Say pong");
    console.log("->", res2.output[0].content[0].text);

    await session.close();
}

main();
```

## Development

The native module is compiled into the root directory.
To re-build after making changes to `src/lib.rs`:

```bash
pnpm run build
```

Check out the [`examples/`](examples/) folder for runnable demonstration scripts.
