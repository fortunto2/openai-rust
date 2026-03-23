# openai-oxide for Node.js

[![npm](https://img.shields.io/npm/v/openai-oxide.svg)](https://www.npmjs.com/package/openai-oxide)
[![npm downloads](https://img.shields.io/npm/dm/openai-oxide.svg)](https://www.npmjs.com/package/openai-oxide)

Native Node.js bindings for [`openai-oxide`](https://github.com/fortunto2/openai-oxide), built with [napi-rs](https://napi.rs/). Also available on [crates.io](https://crates.io/crates/openai-oxide) (Rust) and [PyPI](https://pypi.org/project/openai-oxide/) (Python).

The package exposes the Rust client to Node.js with native streaming and WebSocket support, while keeping release artifacts out of git. Prebuilt binaries are published to npm for the supported targets listed below.

## Features

- Native bindings for chat, responses, streaming, and WebSocket sessions
- Shared Rust core with the main `openai-oxide` crate
- Prebuilt npm artifacts for the main desktop and server targets
- Fast-path APIs for hot loops that return only `text` or `response id`
- Local development and release flow driven by `pnpm`

## Why choose `openai-oxide` on Node?

| Feature | `openai-oxide` | official `openai` SDK |
| :--- | :--- | :--- |
| **WebSocket Responses** | Persistent `wss://` session, reuses TLS for every step | REST-only |
| **Streaming parser** | Zero-copy SSE parser + early function-call parse | HTTP/2 response buffering |
| **Typed Rust core** | Full `Response` struct, hedged requests, parallel fan-outs | Generic JS objects |
| **Hot REST paths** | `createText`, `createStoredResponseId`, `createTextFollowup` avoid JSON bridge | Always serializes `Record<string, any>` |
| **Platform binaries** | Prebuilt `.node` for darwin/linux/windows in npm | Pure JS package |

The official SDK is great for HTTP/REST but does not expose WebSocket streaming or Rust-level hedged/parallel tooling out of the box. If your workload issues quick successive tool calls, streams tokens, or runs inside a WebSocket session, the native bindings keep latency and contention lower while still letting you call the same OpenAI APIs.

## Supported Targets

- macOS `x64`
- macOS `arm64`
- Linux `x64` GNU
- Linux `x64` musl
- Linux `arm64` GNU
- Linux `arm64` musl
- Windows `x64` MSVC

## Install

```bash
npm install openai-oxide
# or
pnpm add openai-oxide
# or
yarn add openai-oxide
```

From the repository for local development:

```bash
cd openai-oxide-node
pnpm install
pnpm build
pnpm test
```

`Client` reads credentials from the same environment variables as the Rust crate, for example `OPENAI_API_KEY`.

## Quick Start

```js
const { Client } = require('openai-oxide')

async function main() {
  const client = new Client()

  const response = await client.createResponse({
    model: 'gpt-4o-mini',
    input: 'Say hello to Node.js from Rust via napi-rs.'
  })

  console.log(response.output?.[0]?.content?.[0]?.text)
}

main().catch((error) => {
  console.error(error)
  process.exitCode = 1
})
```

Examples live in [`examples/`](examples/):

- `examples/01_basic_request.js`
- `examples/02_streaming.js`
- `examples/03_websocket.js`
- `examples/bench_node.js`

## Benchmarks

Benchmarks were run locally against the live OpenAI API with:

```bash
BENCH_ITERATIONS=5 pnpm bench
```

Setup:

- Model: `gpt-5.4`
- Iterations: `5`
- Reported value: median latency
- Comparison target: official [`openai`](https://www.npmjs.com/package/openai) npm SDK

| Test | `openai-oxide` | `openai` | Winner |
| :--- | ---: | ---: | :--- |
| Plain text | 1131ms | 1316ms | `openai-oxide` |
| Structured output | 1467ms | 1244ms | `openai` |
| Function calling | 1103ms | 1151ms | `openai-oxide` |
| Multi-turn (2 reqs) | 1955ms | 2014ms | `openai-oxide` |
| Rapid-fire (5 calls) | 4535ms | 4440ms | `openai` |
| Streaming TTFT | 603ms | 720ms | `openai-oxide` |
| Parallel 3x | 890ms | 947ms | `openai-oxide` |
| WebSocket hot pair | 2359ms | N/A | `openai-oxide` |

Summary:

- `openai-oxide` wins `6` of `8` scenarios
- strongest gains are in plain text, function calling, streaming TTFT, parallel fan-out, and WebSocket reuse
- official `openai` is still faster in this run for structured output and rapid-fire sequential REST calls

For the lowest-overhead REST paths in Node, prefer the fast-path methods:

- `client.createText(model, input, maxOutputTokens?)`
- `client.createStoredResponseId(model, input, maxOutputTokens?)`
- `client.createTextFollowup(model, input, previousResponseId, maxOutputTokens?)`

## Development

Useful commands:

```bash
pnpm install
pnpm build
pnpm test
pnpm bench
pnpm pack:preview
```

`pnpm build` writes the local `.node` binary next to `index.js` for development only. Those generated binaries are ignored by git and are not committed.
`pnpm pack:preview` writes a tarball preview into `.preview/`, which is also ignored by git.

## Release Flow

The repository keeps the Node release separate from the Rust and Python releases.

For the Node package:

1. Keep the Node package version aligned with the Rust crate and Python package version.
2. Push a tag like `node-v0.9.6`.
3. GitHub Actions builds the native addon for each supported target.
4. The Node release workflow assembles platform packages with `napi-rs` and publishes to npm with `pnpm publish`.

Required secrets for npm publishing:

- `NPM_TOKEN`

The workflow uses `pnpm` throughout, publishes with provenance enabled, and keeps platform-specific binaries out of the repository history.
