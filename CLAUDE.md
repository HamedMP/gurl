# gurl — CLAUDE.md

## What is this?

gurl is an open-source, Rust-based HTTP client and runtime for AI agents. "curl for the agentic era." See `specs/001-init/spec.md` for the full vision.

## Project structure

```
gurl/
├── CLAUDE.md              # You are here
├── specs/                 # Feature specifications
│   ├── 001-init/          # Source of truth (full product spec)
│   ├── 002-016/           # Broken-down specs (one per feature)
│   ├── ROADMAP.md         # Dependency graph and phases
│   └── PROGRESS.md        # Current build progress (update this!)
├── crates/
│   ├── gurl-core/         # Core engine: HTTP client, content intelligence, resilience
│   ├── gurl-cli/          # CLI binary (clap)
│   ├── gurl-mcp/          # MCP server (created in spec 008)
│   └── gurl-workflow/     # Workflow engine (created in spec 012)
└── .github/workflows/     # CI
```

## How to work on this project

1. Check `specs/PROGRESS.md` for current status
2. Pick the next incomplete spec from the current phase
3. Read the spec's `spec.md` — it has scope, architecture, and acceptance criteria
4. Reference `specs/001-init/spec.md` for full context when needed
5. Build it, check off acceptance criteria
6. Update `specs/PROGRESS.md` when done
7. Commit with a meaningful message

## Conventions

- **Language:** Rust 2024 edition, MSRV 1.85
- **Async runtime:** tokio
- **HTTP client:** reqwest with rustls-tls
- **CLI framework:** clap (derive API)
- **Serialization:** serde + serde_json
- **Error handling:** thiserror for library errors, anyhow for CLI
- **Binary name:** `gurl`

## Build commands

```bash
cargo build                          # Build everything
cargo run -p gurl-cli -- --version   # Run the CLI
cargo test                           # Run all tests
cargo test -p gurl-core              # Test specific crate
cargo clippy --all-targets           # Lint
```

## Commit style

- Concise messages focused on "why" not "what"
- No co-authored-by lines
- No emojis

## Key design decisions

- **Structured output by default:** Every response is a JSON envelope (spec 001, section 5.3)
- **TTY-aware:** When piped, output just `content.body` (agent-native)
- **Local-first:** Everything works offline. Cloud tier is optional (spec 015)
- **Crate workspace:** Features are split across crates for modularity and compile times
