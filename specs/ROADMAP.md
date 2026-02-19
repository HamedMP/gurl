# gurl Spec Roadmap

> Source of truth: [001-init/spec.md](./001-init/spec.md)

## Dependency graph

```
002-scaffolding
 └── 003-http-client
      ├── 004-html-to-markdown
      │    └── 005-content-detection ─── 006-cli
      │                                    │
      │         008-mcp-server ◄───────────┘ (needs 003, 004, 005)
      │
      ├── 007-resilience
      │    └── 012-workflow-engine
      │
      ├── 009-pdf-extraction
      ├── 010-streaming
      ├── 011-response-diffing
      │    └── 014-watch-monitor
      └── 013-schema-validation

016-distribution (parallel, needs 002)

015-cloud-tier (blocked until core CLI ships)
```

## Phases

### Phase 1 — MVP (ship something agents can use)

| Spec | Name | Effort | Priority |
|------|------|--------|----------|
| 002 | Project Scaffolding | Half day | Must |
| 003 | Core HTTP Client | 1-2 days | Must |
| 004 | HTML to Markdown | 2-3 days | Must |
| 005 | Content Detection | 1 day | Must |
| 006 | CLI Polish | 1 day | Must |
| 008 | MCP Server | 2-3 days | Must |

**Ship v0.1.0 after Phase 1.** This gives agents: HTTP client + structured output + HTML-to-markdown + MCP integration.

### Phase 2 — Power features

| Spec | Name | Effort | Priority |
|------|------|--------|----------|
| 007 | Resilience (Retry/Timeout) | 1 day | High |
| 009 | PDF Extraction | 1-2 days | Medium |
| 010 | Streaming (SSE/WS) | 2 days | High |
| 013 | Schema Validation | 1 day | Medium |
| 016 | Distribution | 1 day | High |

### Phase 3 — Advanced features

| Spec | Name | Effort | Priority |
|------|------|--------|----------|
| 011 | Response Diffing | 1 day | Medium |
| 012 | Workflow Engine | 3-4 days | Medium |
| 014 | Watch/Monitor | 1 day | Low |

### Phase 4 — Cloud

| Spec | Name | Effort | Priority |
|------|------|--------|----------|
| 015 | Cloud Tier | 2-3 weeks | Deferred |

## How to use these specs

1. Pick a spec from the current phase
2. Read it, then reference back to 001-init for full context
3. Build it, check off the acceptance criteria
4. Move to the next spec

Each spec is designed to be self-contained enough that you can hand it to an AI agent (or tackle it yourself) without re-reading the entire 001 doc.
