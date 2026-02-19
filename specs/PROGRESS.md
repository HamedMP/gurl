# Build Progress

## Phase 1 — MVP

| Spec | Name | Status | Notes |
|------|------|--------|-------|
| 002 | Project Scaffolding | done | Cargo workspace, CI, clap CLI skeleton |
| 003 | Core HTTP Client | done | reqwest, envelope, timing, all methods, --raw |
| 004 | markitdown-rs | in-progress | Full Rust port of MarkItDown — Phase A (foundation + HTML) |
| 005 | Content Detection | pending | |
| 006 | CLI Polish | pending | |
| 008 | MCP Server | pending | |

### 004 markitdown-rs sub-progress

| Phase | Converters | Status |
|-------|-----------|--------|
| A — Foundation + HTML | trait, detection, PlainText, HTML, CSV | in-progress |
| B — Documents | PDF, DOCX, XLSX/XLS, EPUB, RSS | pending |
| C — Media + Specialty | Image, Audio, Outlook, Jupyter, PPTX, ZIP, Wikipedia | pending |
| D — Polish | gurl integration, standalone CLI, tests, benchmarks | pending |

## Phase 2 — Power Features

| Spec | Name | Status | Notes |
|------|------|--------|-------|
| 007 | Resilience | pending | |
| 010 | Streaming | pending | |
| 013 | Schema Validation | pending | |
| 016 | Distribution | pending | |

## Phase 3 — Advanced

| Spec | Name | Status | Notes |
|------|------|--------|-------|
| 011 | Response Diffing | pending | |
| 012 | Workflow Engine | pending | |
| 014 | Watch/Monitor | pending | |

## Phase 4 — Cloud

| Spec | Name | Status | Notes |
|------|------|--------|-------|
| 015 | Cloud Tier | blocked | Ship core CLI first |
