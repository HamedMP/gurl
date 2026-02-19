# 010 — Streaming: SSE & WebSocket

> **Parent spec:** [001-init](../001-init/spec.md) — Sections 6.2 F2.4
> **Status:** Ready
> **Dependencies:** 003-http-client
> **Estimated effort:** 2 days

---

## Goal

Support Server-Sent Events (SSE) and WebSocket connections. Critical for agents that interact with LLM APIs (OpenAI, Anthropic) and real-time data feeds.

## Scope

### In scope

- SSE client: connect to SSE endpoint, output events as they arrive
- WebSocket client: connect, send messages, receive messages
- `gurl stream <url> --sse` for SSE connections
- `gurl ws <url>` for WebSocket connections
- `--send` flag for WebSocket to send initial message
- Streaming JSON output: one JSON object per event/message
- LLM streaming helper: `--stream` flag on POST that handles OpenAI-style SSE
- Add `stream` and `ws` subcommands to CLI

### Out of scope

- WebSocket server mode
- Custom WebSocket subprotocols
- Reconnection logic for SSE (nice-to-have, but not MVP)
- Binary WebSocket frames

## SSE output format

Each SSE event is emitted as a JSON line:

```jsonl
{"event": "message", "data": "Hello", "id": "1", "timestamp": "2026-02-19T12:00:01Z"}
{"event": "message", "data": "World", "id": "2", "timestamp": "2026-02-19T12:00:02Z"}
{"event": "done", "data": "[DONE]", "id": "3", "timestamp": "2026-02-19T12:00:03Z"}
```

## WebSocket output format

```jsonl
{"direction": "sent", "data": "{\"subscribe\": \"updates\"}", "timestamp": "..."}
{"direction": "received", "data": "{\"type\": \"update\", ...}", "timestamp": "..."}
```

## LLM streaming mode

For OpenAI-compatible streaming APIs:

```bash
gurl post https://api.openai.com/v1/chat/completions \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  --json '{"model": "gpt-4", "messages": [{"role": "user", "content": "Hi"}], "stream": true}' \
  --stream
```

The `--stream` flag on POST:
1. Detects SSE in the response
2. Parses `data:` lines
3. Outputs each chunk's content delta (not the raw SSE wrapper)

## Dependencies (crates)

- `tokio-tungstenite` for WebSocket
- `eventsource-stream` or custom SSE parser
- `futures-util` for stream processing

## Acceptance criteria

- [ ] `gurl stream https://sse-test-endpoint --sse` outputs events as JSON lines
- [ ] `gurl ws wss://echo.websocket.org --send '{"hello": "world"}'` connects and echoes
- [ ] `--stream` on a POST with SSE response outputs content deltas
- [ ] Ctrl+C cleanly disconnects SSE/WebSocket connections
- [ ] Connection errors produce structured error output
- [ ] `gurl stream` and `gurl ws` appear in `--help`
