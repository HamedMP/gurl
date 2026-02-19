# 002 — Project Scaffolding

> **Parent spec:** [001-init](../001-init/spec.md)
> **Status:** Ready
> **Dependencies:** None
> **Estimated effort:** Half day

---

## Goal

Set up the Cargo workspace, crate structure, CI, and build pipeline so that `gurl --version` compiles and runs on macOS, Linux, and Windows.

## Scope

### In scope

- Cargo workspace with crate layout from spec 001 section 8.2
- Initial crates: `gurl-core`, `gurl-cli` (empty shells with `lib.rs`/`main.rs`)
- `clap` CLI skeleton in `gurl-cli` with version/help
- GitHub Actions CI: build + test on Linux/macOS/Windows
- Cross-compilation targets: `x86_64-apple-darwin`, `aarch64-apple-darwin`, `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `x86_64-pc-windows-msvc`
- MIT LICENSE file
- `.gitignore` for Rust
- Basic `Cargo.toml` workspace configuration

### Out of scope

- Homebrew tap, npm package, install script (see 016)
- Any actual HTTP functionality
- README beyond a one-liner (will evolve with features)

## Crate structure

```
gurl/
├── Cargo.toml              # Workspace root
├── LICENSE
├── .gitignore
├── crates/
│   ├── gurl-core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   └── gurl-cli/
│       ├── Cargo.toml
│       └── src/
│           └── main.rs
└── .github/
    └── workflows/
        └── ci.yml
```

Additional crates (`gurl-mcp`, `gurl-workflow`) are created in their respective specs when needed. Don't create empty shells for them now.

## Acceptance criteria

- [ ] `cargo build` succeeds from workspace root
- [ ] `cargo run -p gurl-cli -- --version` prints `gurl 0.1.0`
- [ ] `cargo run -p gurl-cli -- --help` shows placeholder command structure
- [ ] `cargo test` passes (even if no real tests yet)
- [ ] CI builds green on all three OS targets

## Key decisions

- **Edition:** Rust 2024
- **MSRV:** 1.85 (latest stable at time of writing)
- **Binary name:** `gurl` (verify crates.io availability before publish)

## CLI skeleton

The `clap` setup should define the subcommand structure from spec 001 section 8.3, but commands can just print "not yet implemented" for now. This establishes the public interface early.

```
gurl <command> [options] [url]

Commands:
  get, post, put, patch, delete, head    HTTP methods
  fetch                                   Smart fetch
  mcp                                     Start MCP server
  version                                 Show version
```

Phase 2+ commands (`chain`, `diff`, `watch`, `stream`, `search`, `crawl`) should NOT be added yet. Add them in their respective specs.
