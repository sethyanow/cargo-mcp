---
id: cm-qnh
title: Add cargo_metadata tool
status: open
type: feature
priority: 2
phase: design
---


## Context

`cargo metadata --format-version=1` returns structured JSON about the
project: workspace members, dependency graph, features, resolved
versions, target directory, workspace root. Agents constantly need
this information to understand project structure before making changes.

Currently no tool exposes it. Agents have no way to answer "what crates
are in this workspace?", "what features does this crate have?", or
"where is the target directory?" without shell access.

Also used by cm-x8c (managed target directory) for resolving
`target_directory`.

## Requirements

R1. A `cargo_metadata` tool MUST exist that runs
    `cargo metadata --format-version=1` and returns the JSON output.

R2. The tool MUST accept `package` (optional, filter to workspace
    member), `toolchain`, and `cargo_env` params for consistency
    with other tools.

R3. Output MUST be the raw JSON from cargo metadata — returned directly,
    NOT passed through `execute_cargo_command`. The metadata JSON is the
    response. No wrapper, no nesting.

R4. The tool does NOT require `set_working_directory` to have been
    called if CWD auto-detect (cm-dvs) is active — but must work
    with the current working directory mechanism.

## Scope

### Files that change

| File | What changes |
|------|-------------|
| `src/tools/cargo_metadata.rs` | New tool: runs `cargo metadata --format-version=1` |
| `src/tools.rs` | Register `CargoMetadata` in `tools!` macro |

### Design notes
- `cargo metadata` outputs to stdout, not stderr. The raw JSON is the
  useful output — don't wrap it in the execute_cargo_command prose
  format.
- Bypasses `execute_cargo_command` — uses the timeout wrapper directly
  and returns stdout as the response. No JSON-in-JSON nesting.
- `--no-deps` flag is useful to skip dependency resolution (faster).
  Expose as a param.

## Success Criteria

- [ ] SC1: `cargo_metadata {}` returns valid JSON with `workspace_root`,
      `target_directory`, `packages`, and `resolve` fields
- [ ] SC2: Output is raw JSON, not prose-wrapped
- [ ] SC3: `--no-deps` param skips dependency resolution when set
- [ ] SC4: All tests pass, clippy clean, fmt clean

## Anti-Patterns (FORBIDDEN)

- NO parsing or filtering the metadata JSON (return raw)
- NO routing through `execute_cargo_command` (no JSON-in-JSON)
