# 012 — Workflow Engine (Request Chaining)

> **Parent spec:** [001-init](../001-init/spec.md) — Sections 6.2 F2.1, 14.5
> **Status:** Ready
> **Dependencies:** 003-http-client, 007-resilience
> **Estimated effort:** 3-4 days

---

## Goal

YAML-based workflow definitions that let agents chain HTTP requests: authenticate, capture tokens, use them in subsequent requests, loop over results, and assert conditions. This is `gurl chain <workflow.yaml>`.

## Scope

### In scope

- New crate: `gurl-workflow`
- YAML workflow parser
- Step execution: sequential by default
- Variable capture from responses: JSONPath, header, regex
- Template variables in URLs, headers, bodies: `{{step_id.variable}}`
- Environment variable interpolation: `{{env.API_KEY}}`
- Assertions: status code, body contains, JSONPath conditions
- `for_each` iteration over arrays
- `parallel` execution within `for_each`
- `condition` for conditional step execution
- `delay` between steps
- Step-level retry/timeout (inherits from 007)
- Add `chain` subcommand to CLI

### Out of scope

- Workflow composition (importing other workflows)
- GUI/visual workflow builder
- Persistent workflow state / resume
- Webhook triggers

## Crate structure

```
crates/gurl-workflow/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── parser.rs       # YAML -> Workflow struct
    ├── executor.rs     # Run workflow steps
    ├── capture.rs      # Variable capture (JSONPath, regex, header)
    ├── template.rs     # Variable interpolation in strings
    ├── assertion.rs    # Assert conditions
    └── types.rs        # Workflow, Step, Capture, Assert types
```

## Workflow output

```json
{
  "gurl": "0.1.0",
  "workflow": {
    "name": "Authenticated API fetch",
    "steps": [
      {
        "id": "login",
        "status": "passed",
        "response_status": 200,
        "captures": { "token": "eyJ..." },
        "timing_ms": 234
      },
      {
        "id": "fetch_data",
        "status": "passed",
        "response_status": 200,
        "captures": { "items": "[...]" },
        "assertions": [
          { "type": "status", "expected": 200, "actual": 200, "passed": true }
        ],
        "timing_ms": 156
      }
    ],
    "summary": {
      "total_steps": 2,
      "passed": 2,
      "failed": 0,
      "total_ms": 390
    }
  }
}
```

## YAML syntax (subset to implement)

```yaml
name: "Auth flow"
steps:
  - id: login
    method: POST
    url: https://api.example.com/auth
    json:
      email: "{{env.EMAIL}}"
      password: "{{env.PASSWORD}}"
    capture:
      token: jsonpath $.access_token

  - id: get_users
    method: GET
    url: https://api.example.com/users
    headers:
      Authorization: "Bearer {{login.token}}"
    assert:
      status: 200
```

See spec 001 section 14.5 for the full syntax reference.

## Dependencies (crates)

- `serde_yaml` for YAML parsing
- `jsonpath_lib` or `serde_json_path` for JSONPath queries
- `regex` for regex capture

## Acceptance criteria

- [ ] `gurl chain workflow.yaml` executes steps sequentially
- [ ] Variables captured from one step are available in the next
- [ ] `{{env.VAR}}` interpolates environment variables
- [ ] `for_each` iterates over captured arrays
- [ ] `parallel: N` runs iterations concurrently
- [ ] `assert.status: 200` fails the workflow if status doesn't match
- [ ] Failed assertions produce clear error output with step ID
- [ ] Workflow summary shows pass/fail counts and total timing
- [ ] Invalid YAML produces a helpful parse error, not a panic
