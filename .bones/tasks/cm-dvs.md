---
id: cm-dvs
title: Auto-detect working directory from CWD at startup
status: open
type: task
priority: 2
phase: design
---

## Context

Every agent session starts with a mandatory `set_working_directory` call
before any tool works. `ensure_rust_project` errors with "No working
directory set" otherwise. This is a wasted round-trip — the server
process already has a CWD, and each agent gets its own server instance.

If the server's CWD contains a `Cargo.toml` (or is inside a cargo
workspace), it should just use that at startup. `set_working_directory`
remains available for agents that need to switch, but the common case
of "I'm already in the project" should work without it.

## Requirements

R1. On startup, `CargoTools::new()` MUST check if `std::env::current_dir()`
    contains a `Cargo.toml`. If so, set it as the working directory
    automatically.

R2. If CWD is not a Rust project, no working directory is set —
    existing behavior preserved (`set_working_directory` still required).

R3. `set_working_directory` MUST still work to override the auto-detected
    directory. Not removed, not changed.

R4. The INSTRUCTIONS string SHOULD be updated to reflect that
    `set_working_directory` is only needed when the server wasn't
    launched from a project directory.

## Scope

### Files that change

| File | What changes |
|------|-------------|
| `src/state.rs` | `CargoTools::new()` probes CWD for `Cargo.toml`, calls `set_working_directory` if found |
| `src/main.rs` | Update INSTRUCTIONS string to note auto-detection |

### Files that DON'T change
- `src/tools/set_working_directory.rs` — tool stays as-is
- All other tool files — they call `ensure_rust_project` which works the same

## Success Criteria

- [ ] SC1: Server launched from a Rust project dir can run tools without
      `set_working_directory` call
- [ ] SC2: Server launched from a non-project dir still requires
      `set_working_directory` (no regression)
- [ ] SC3: `set_working_directory` overrides the auto-detected dir
- [ ] SC4: All tests pass, clippy clean, fmt clean

## Anti-Patterns (FORBIDDEN)

- NO walking up the directory tree to find Cargo.toml (CWD only —
  predictable, and MCP clients launch the server from the project root)
- NO making `set_working_directory` tool deprecated or hidden
- NO persisting the auto-detected dir to disk (in-memory only, per existing design)
