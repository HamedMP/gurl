# 006 — CLI Polish & Configuration

> **Parent spec:** [001-init](../001-init/spec.md) — Sections 8.3, 8.4
> **Status:** Ready
> **Dependencies:** 003-http-client, 004-html-to-markdown, 005-content-detection
> **Estimated effort:** 1 day

---

## Goal

Polish the CLI experience: proper error messages, colored output for terminals, configuration file support, and the `--quiet` mode that makes gurl truly agent-native (output just the content body when piped).

## Scope

### In scope

- TTY detection: when stdout is a pipe, default to `--quiet` (body only)
- `--quiet` / `-q`: output only `content.body` (no envelope)
- Colored/formatted terminal output when interactive
- Configuration file: `~/.config/gurl/config.yaml`
- `gurl config` subcommand (show current config, set values)
- Proper error formatting (structured JSON errors, not panics)
- `--output` / `-o` flag to save response body to file
- User-Agent configuration via config file
- Exit codes: 0 for success, 1 for HTTP errors, 2 for connection/network errors

### Out of scope

- Cloud-specific config (`api_key`, `endpoint`) — see 015
- Search provider config — later
- Proxy config — later

## TTY-aware output

This is a key agent-native design decision:

```bash
# Interactive terminal — full envelope, pretty-printed
gurl get https://api.example.com/users
# { "gurl": "0.1.0", "response": {...}, "content": {...} }

# Piped to another command — just the body
gurl get https://api.example.com/users | jq '.items'
# outputs content.body directly

# Force behavior
gurl get https://api.example.com/users --quiet   # always body-only
gurl get https://api.example.com/users --format json  # always envelope
```

## Config file

```yaml
# ~/.config/gurl/config.yaml
defaults:
  format: json
  timeout: 30s
  connect_timeout: 5s
  user_agent: "gurl/0.1.0"

content:
  html_to_markdown: true
  extract_metadata: true
  include_links: true
  include_images: true
  max_content_length: 100000
```

## Error envelope

```json
{
  "gurl": "0.1.0",
  "error": {
    "code": "connection_failed",
    "message": "Failed to connect to host: Connection refused",
    "url": "https://nonexistent.example.com",
    "timestamp": "2026-02-19T12:00:00Z"
  }
}
```

## Acceptance criteria

- [ ] Piping gurl output to `jq` works cleanly (no envelope wrapping)
- [ ] Interactive terminal shows formatted envelope
- [ ] `gurl get https://bad.host` shows structured error, exits with code 2
- [ ] `gurl get https://httpbin.org/status/404` exits with code 1
- [ ] `gurl get https://httpbin.org/get` exits with code 0
- [ ] `~/.config/gurl/config.yaml` is read if it exists
- [ ] `--output report.json` saves response body to file
- [ ] `gurl config` shows current resolved config
