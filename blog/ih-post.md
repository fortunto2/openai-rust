---
platform: Indie Hackers
title: "I built one Rust crate and now it powers all my products — coding agent, iOS video app, voice assistant"
---

Hey IH community!

I'm a solo founder building multiple products at once. Instead of writing separate AI integrations for each one, I built a shared Rust core that compiles everywhere. Wanted to share how this "build the tooling first" approach is working out.

**The Backstory**

I was building a realtime voice assistant and needed low-latency OpenAI calls. Python's garbage collection was causing audible glitches in the audio stream. I wrote a small Rust client to hold a WebSocket open. Then I needed the same thing for a coding agent. Then for an iOS app. At some point I looked at what I had and realized it was a full OpenAI SDK running on 5 platforms.

**What I Built**

[openai-oxide](https://github.com/fortunto2/openai-oxide) — a Rust OpenAI client with bindings for Node.js, Python, WASM (browsers), and iOS (via UniFFI → Swift). One codebase, same behavior everywhere.

On top of it sits [sgr-agent](https://github.com/fortunto2/rust-code/tree/master/crates/sgr-agent) — my agent framework for structured outputs, tool calling loops, and multi-provider support (OpenAI, OpenRouter, Gemini, Ollama through one client).

Three products use this stack:
- **Coding agent** ([rust-code](https://github.com/fortunto2/rust-code)) — TUI that reads your codebase, plans edits, calls tools, iterates. Terminal today, browser (WASM) next.
- **Video montage app** (iOS, private) — analyzes camera roll with Apple Vision, uses an AI agent to select scenes and plan voiceover. Rust agent compiled to static lib, Swift bindings auto-generated via UniFFI.
- **Voice assistant** — persistent WebSocket to OpenAI, connection pooling, sub-second turns.

**Key Numbers**

- Time to build the SDK: 5 days initial, ~3 weeks to production quality
- 500+ commits, 193 tests, 1100+ typed API models
- Published on: crates.io, npm, PyPI
- Downloads: 287 (Rust), 522/mo (npm), 1,685/mo (Python)
- Stars: 14 (just getting started)
- Featured in: This Week in Rust #645
- Revenue: pre-revenue (this is infrastructure, not a SaaS)

**The Solo Factory Approach**

I'm building a [solo-factory](https://github.com/fortunto2/solo-factory) toolkit with Claude Code skills for the full startup lifecycle — planning, building, deploying, reviewing. openai-oxide is the AI infrastructure layer. Every new product I start gets the same agent framework and the same LLM client.

The idea: don't build one startup. Build the tooling that lets you ship startups faster, then use it to try many ideas.

**Lessons Learned**

1. **Multi-platform constraints improve your code.** When the same crate has to work on iOS (no runtime customization), in WASM (no threads, no filesystem), and in Python (GIL), you end up with cleaner abstractions than if you only targeted one platform.

2. **Benchmarks will make you look like a liar if you're not careful.** I spent a week claiming "40% faster" based on n=5 measurements. Proper statistical analysis (Welch's t-test) showed most differences were API noise. Honest benchmarks are harder but build trust.

3. **Auto-sync beats manual maintenance.** OpenAI has 1100+ types that change constantly. I wrote a Python→Rust code generator that reads their SDK and produces Rust types. One command to stay current. Manually tracking API changes would have killed the project.

4. **UniFFI is underrated for iOS.** Defining types in Rust and getting native Swift types automatically is a huge productivity win. The agent loop runs in Rust, Swift handles UI and Apple frameworks. Clean separation.

**What's Next**

- Android bindings (UniFFI → Kotlin)
- Browser-based coding agent (same Rust code compiled to WASM)
- More solo-factory skills for automated launches
- Building karma on HN so I can actually post there (got auto-flagged today with karma 1)

**The Ask**

Curious about two things:
- Anyone else doing the "one core, thin bindings" approach for developer tools? How's the maintenance burden long-term?
- For those building multiple products solo — do you share infrastructure between them, or keep them fully independent?

Links:
- GitHub: [openai-oxide](https://github.com/fortunto2/openai-oxide)
- Coding agent: [rust-code](https://github.com/fortunto2/rust-code)
- Solo factory: [solo-factory](https://github.com/fortunto2/solo-factory)
- Dev.to article: [I ported the OpenAI Python SDK to Rust in 5 days](https://dev.to/fortunto2/squeezing-every-millisecond-from-the-openai-api-in-rust-4b11)

Happy to answer questions about the Rust/Swift/WASM setup!
