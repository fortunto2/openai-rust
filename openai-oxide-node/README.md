# openai-oxide-node

Native Node.js bindings for [`openai-oxide`](https://github.com/fortunto2/openai-rust), built with [napi-rs](https://napi.rs/).

The package exposes the Rust client to Node.js with native streaming and WebSocket support, while keeping release artifacts out of git. Prebuilt binaries are published to npm for the supported targets listed below.

## Features

- Native bindings for chat, responses, streaming, and WebSocket sessions
- Shared Rust core with the main `openai-oxide` crate
- Prebuilt npm artifacts for the main desktop and server targets
- Local development and release flow driven by `pnpm`

## Supported Targets

- macOS `x64`
- macOS `arm64`
- Linux `x64` GNU
- Linux `x64` musl
- Linux `arm64` GNU
- Linux `arm64` musl
- Windows `x64` MSVC

## Install

From npm:

```bash
pnpm add openai-oxide-node
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
const { Client } = require('openai-oxide-node')

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

## Development

Useful commands:

```bash
pnpm install
pnpm build
pnpm test
pnpm pack:preview
```

`pnpm build` writes the local `.node` binary next to `index.js` for development only. Those generated binaries are ignored by git and are not committed.
`pnpm pack:preview` writes a tarball preview into `.preview/`, which is also ignored by git.

## Release Flow

The repository keeps the Node release separate from the Rust and Python releases.

For the Node package:

1. Keep the Node package version aligned with the Rust crate and Python package version.
2. Push a tag like `node-v0.9.4`.
3. GitHub Actions builds the native addon for each supported target.
4. The Node release workflow assembles platform packages with `napi-rs` and publishes to npm with `pnpm publish`.

Required secrets for npm publishing:

- `NPM_TOKEN`

The workflow uses `pnpm` throughout, publishes with provenance enabled, and keeps platform-specific binaries out of the repository history.
