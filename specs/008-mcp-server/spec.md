# 008 — MCP Server

> **Parent spec:** [001-init](../001-init/spec.md) — Sections 6.1 F1.3, 14.4
> **Status:** Ready
> **Dependencies:** 003-http-client, 004-html-to-markdown, 005-content-detection
> **Estimated effort:** 2-3 days

---

## Goal

Implement `gurl mcp` — a Model Context Protocol server that exposes gurl's capabilities as tools for AI agents (Claude Desktop, Cursor, Claude Code, etc.). This is the primary integration surface for agents.

## Scope

### In scope

- MCP server over stdio transport (primary)
- Tool definitions: `gurl_fetch`, `gurl_api`
- MCP protocol compliance: initialize, list_tools, call_tool
- Proper error handling in MCP responses
- New crate: `gurl-mcp`

### Out of scope (add in later specs)

- `gurl_search` tool (needs search backend — later)
- `gurl_extract` tool (needs CSS selectors — later)
- `gurl_diff` tool (see 011)
- HTTP/SSE transport for MCP (stdio is sufficient for now)
- Resource and prompt MCP primitives (tools only for now)

## Crate structure

```
crates/gurl-mcp/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── server.rs        # MCP server loop (stdio read/write)
    ├── protocol.rs      # MCP message types (JSON-RPC)
    └── tools/
        ├── mod.rs
        ├── fetch.rs     # gurl_fetch tool
        └── api.rs       # gurl_api tool
```

## Tool definitions

### gurl_fetch

```json
{
  "name": "gurl_fetch",
  "description": "Fetch a URL and return its content as clean, LLM-friendly text. Automatically converts HTML to markdown, handles JSON, PDFs, and other content types.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "url": { "type": "string", "description": "The URL to fetch" },
      "format": { "type": "string", "enum": ["markdown", "json", "html", "raw"], "default": "markdown" },
      "headers": { "type": "object", "description": "Custom request headers" },
      "timeout": { "type": "integer", "description": "Timeout in seconds", "default": 30 }
    },
    "required": ["url"]
  }
}
```

### gurl_api

```json
{
  "name": "gurl_api",
  "description": "Make an HTTP API request with full control over method, headers, and body. Returns structured response with status, headers, and parsed body.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "method": { "type": "string", "enum": ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"] },
      "url": { "type": "string" },
      "headers": { "type": "object" },
      "body": { "type": "string", "description": "Request body as string" },
      "json": { "type": "object", "description": "JSON body (auto-sets Content-Type)" }
    },
    "required": ["method", "url"]
  }
}
```

## MCP protocol notes

- MCP uses JSON-RPC 2.0 over stdio
- Server must handle: `initialize`, `tools/list`, `tools/call`
- Consider using an existing MCP SDK crate if one exists and is mature enough, otherwise implement the thin protocol layer directly — it's straightforward JSON-RPC
- Tool results should return content as `text` type (the markdown/JSON string)

## Integration test approach

- Test MCP server by spawning `gurl mcp` as a subprocess
- Send JSON-RPC messages to stdin, read responses from stdout
- Verify tool listing, successful fetch, error handling

## Acceptance criteria

- [ ] `gurl mcp` starts and responds to `initialize` request
- [ ] `tools/list` returns `gurl_fetch` and `gurl_api` tool definitions
- [ ] Calling `gurl_fetch` with a URL returns markdown content
- [ ] Calling `gurl_api` with method + URL returns structured response
- [ ] Invalid tool calls return proper MCP error responses
- [ ] Server handles malformed JSON-RPC gracefully (doesn't crash)
- [ ] Works with Claude Desktop (manual test: add to config, verify agent can use it)

## Claude Desktop config

```json
{
  "mcpServers": {
    "gurl": {
      "command": "gurl",
      "args": ["mcp"]
    }
  }
}
```
