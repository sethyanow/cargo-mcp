---
id: cm-0au
title: Add cargo_fix tool
status: open
type: feature
priority: 2
phase: design
---

## Context

`cargo fix` auto-applies compiler and clippy suggestions. Instead of an
agent reading a diagnostic, understanding the suggestion, and manually
editing the file, one tool call fixes it. Pairs naturally with cm-szb
(structured diagnostics): diagnose with `--message-format=json`, then
auto-fix with `cargo fix`.

`cargo fix` modifies files in-place. This is the intended behavior —
the agent is asking cargo to apply its own suggestions.

## Requirements

R1. A `cargo_fix` tool MUST exist that runs `cargo fix` with
    appropriate flags.

R2. The tool MUST support:
    - `package` — target specific package
    - `clippy` — when true, apply clippy suggestions
      (`--clippy` flag or `cargo clippy --fix`)
    - `allow_dirty` — allow fixes in dirty working tree (default true,
      since agent workflows often have uncommitted changes)
    - `allow_staged` — allow fixes with staged changes (default true)
    - `toolchain`, `cargo_env`, `extra_args` — standard params

R3. Default behavior: `cargo fix --allow-dirty --allow-staged`.
    Agents work in dirty trees by definition.

## Scope

### Files that change

| File | What changes |
|------|-------------|
| `src/tools/cargo_fix.rs` | New tool following existing pattern |
| `src/tools.rs` | Register `CargoFix` in `tools!` macro |

### Design notes
- `cargo fix` vs `cargo clippy --fix`: both apply suggestions. The
  `--clippy` flag on `cargo fix` is experimental. Simpler to have one
  tool that covers both.
- `cargo fix` requires `--allow-dirty` and `--allow-staged` to run
  in a non-clean working tree. Default both to true for agent use.
- Standard `build_args()` / `execute_cargo_command` pattern applies.

## Success Criteria

- [ ] SC1: `cargo_fix {}` runs `cargo fix --allow-dirty --allow-staged`
- [ ] SC2: `cargo_fix { clippy: true }` applies clippy suggestions
- [ ] SC3: `cargo_fix { package: "my-lib" }` scopes to specific package
- [ ] SC4: All tests pass, clippy clean, fmt clean

## Anti-Patterns (FORBIDDEN)

- NO defaulting `allow_dirty`/`allow_staged` to false (agents always
  have dirty trees)
- NO separate tool for clippy fixes vs compiler fixes (one tool)
