---
id: cm-ip0
title: Switch output format from prose to JSON
status: open
type: task
priority: 2
phase: design
---


## Context

All tool responses use prose strings with emojis and labels. Two sources:
- `execute_cargo_command` (lines 122-158 of `cargo_utils.rs`) — used by
  all 12 cargo tools
- `set_working_directory` (lines 56-67) — its own inline formatting

This output is never read by a human — the consumer is always an AI
agent via MCP. The current format wastes tokens on decoration and forces
agents to parse prose to extract exit codes and distinguish stdout from
stderr.

## Requirements

R1. `execute_cargo_command` MUST return a JSON string instead of
    formatted prose. The JSON object MUST contain: `command` (string),
    `dir` (string), `exit_code` (integer), `success` (boolean),
    `stdout` (string), `stderr` (string).

R2. The `format_command` and `shell_escape` helpers MUST be retained —
    `command` field uses the same formatted command string.

R3. Empty stdout/stderr MUST be represented as empty strings, not
    omitted. Agents should not need null-checks.

R4. Timeout errors remain as `Err` (anyhow bail) — not JSON. The
    distinction between "command ran and failed" (Ok with exit_code != 0)
    and "command could not complete" (Err) is meaningful.

R5. Use `serde_json` for JSON construction (already a dependency).
    Build a `serde_json::json!` value and serialize — no manual escaping.

R6. `set_working_directory` MUST return JSON instead of emoji prose.
    Fields: `dir` (string), `rust_project` (boolean — whether
    Cargo.toml was found).

## Scope

### Files that change

| File | What changes |
|------|-------------|
| `src/tools/cargo_utils.rs` | Lines 122-158: replace prose formatting with JSON construction |
| `src/tools/cargo_utils.rs` | 3 tests: update assertions from prose markers to JSON fields |
| `src/tools/set_working_directory.rs` | Lines 56-67: replace emoji prose with JSON |

### Files that DON'T change
- `src/tests.rs` — 71 tests assert on `build_args()`, never on output format
- Other `src/tools/cargo_*.rs` — they delegate to `execute_cargo_command`
- `src/main.rs` — no output formatting

### Impact
- Two formatting sites: `execute_cargo_command` and `set_working_directory`
- 3 tests in cargo_utils.rs need updated assertions
- 0 callers of execute_cargo_command need changes

## Success Criteria

- [ ] SC1: `execute_cargo_command` output is valid JSON parseable by any JSON parser
- [ ] SC2: JSON contains all six fields: `command`, `dir`, `exit_code`, `success`, `stdout`, `stderr`
- [ ] SC3: No emojis or prose labels in any tool output
- [ ] SC7: `set_working_directory` returns JSON with `dir` and `rust_project` fields
- [ ] SC4: `exit_code` is an integer (not a string), `success` is a boolean
- [ ] SC5: Timeout path still returns `Err`, not JSON with a timeout field
- [ ] SC6: All tests pass, clippy clean, fmt clean

## Anti-Patterns (FORBIDDEN)

- NO manual JSON string construction (use serde_json, it's already a dep)
- NO wrapping output in MCP-specific envelope (mcplease handles that)
- NO changing the function signature — return type stays `Result<String>`
- NO truncating stdout/stderr (agent decides what to use)
