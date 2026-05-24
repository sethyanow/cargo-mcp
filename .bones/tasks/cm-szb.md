---
id: cm-szb
title: Structured diagnostics via --message-format=json
status: open
type: feature
priority: 1
phase: design
---

## Context

`cargo check`, `cargo clippy`, and `cargo build` all support
`--message-format=json`, which emits one JSON object per line with
structured diagnostic info: file path, line/column spans, severity,
error code, human message, and machine-applicable suggested fixes.

Currently the server returns raw stderr prose. Agents parse error
messages with regex or heuristics, miss multi-line spans, and can't
programmatically apply suggested fixes. Structured diagnostics let
agents jump to exact locations and understand fix applicability
without guessing.

This pairs with cm-0au (`cargo_fix`) — diagnostics identify problems,
fix auto-applies them.

## Requirements

R1. `cargo_check`, `cargo_clippy`, and `cargo_build` MUST support a
    `message_format` param. Default is `"json"` — passes
    `--message-format=json` to cargo automatically.

R2. When `message_format` is `"json"`, the tool output MUST return
    the raw JSON lines from cargo, not the prose-formatted output.
    Each line is a self-contained JSON object — return them as-is.

R3. `message_format` can be set to `"human"` to get cargo's default
    prose output. Agents are the consumer — structured is the default,
    prose is the escape hatch.

R4. The JSON output MUST NOT be wrapped in additional formatting.
    Raw cargo JSON only.

## Scope

### Files that change

| File | What changes |
|------|-------------|
| `src/tools/cargo_check.rs` | Add `message_format` param, pass `--message-format=json` when set |
| `src/tools/cargo_clippy.rs` | Same |
| `src/tools/cargo_build.rs` | Same |

### Files that DON'T change
- `src/tools/cargo_utils.rs` — `execute_cargo_command` returns whatever
  cargo outputs; no special JSON handling needed
- Other tools — `cargo test`, `cargo fmt`, etc. don't support
  `--message-format`

### Design notes
- Cargo's JSON diagnostic format is one JSON object per line (not a
  JSON array). Each object has `reason` field: `"compiler-message"`,
  `"compiler-artifact"`, `"build-script-executed"`, `"build-finished"`.
- Agent-relevant objects are `reason: "compiler-message"` which contain
  `message.spans[]` with file, line, column, and `message.children[]`
  with suggestions.
- `--message-format=json` redirects diagnostics to stdout (not stderr).
  Human-readable output still goes to stderr.

## Success Criteria

- [ ] SC1: `cargo_check {}` (default) returns JSON lines with structured diagnostics
- [ ] SC2: Each line is valid JSON with `reason` field
- [ ] SC3: `cargo_clippy {}` (default) returns structured clippy diagnostics
      with spans and suggestions
- [ ] SC4: `cargo_check { message_format: "human" }` returns prose output
- [ ] SC5: All tests pass, clippy clean, fmt clean

## Anti-Patterns (FORBIDDEN)

- NO parsing or transforming the JSON cargo emits (pass through raw)
- NO defaulting to prose output (agents are the consumer — JSON default)
- NO adding `message_format` to tools that don't support it (test, fmt, etc.)
