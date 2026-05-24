---
id: cm-k0q
title: Add cargo_init and cargo_new tools
status: open
type: feature
priority: 3
phase: design
---

## Context

Agents can't create new Rust projects or add crates to a workspace.
`cargo init` initializes a project in an existing directory, `cargo new`
creates a new directory with a project. Both are standard scaffolding
operations agents need for greenfield work.

## Requirements

R1. A `cargo_init` tool MUST exist that runs `cargo init` in the
    working directory (or a specified path).

R2. A `cargo_new` tool MUST exist that runs `cargo new <name>`.

R3. `cargo_new` MUST support:
    - `name` — project name (required, creates directory)
    - `lib` — create a library instead of binary (`--lib`)
    - `edition` — Rust edition (e.g., `"2024"`)
    - `vcs` — version control init (`"git"`, `"none"`, etc.) — agents
      creating crates inside existing repos usually want `--vcs none`
    - `toolchain`, `cargo_env`, `extra_args` — standard params

R4. `cargo_init` MUST support:
    - `lib` — create a library instead of binary (`--lib`)
    - `edition` — Rust edition
    - `vcs` — version control init
    - `toolchain`, `cargo_env`, `extra_args` — standard params
    - Name is inferred from the directory (cargo init behavior).

R5. `cargo_new` does NOT require `set_working_directory` to have been
    called first — it creates the directory. `cargo_init` uses the
    current working directory.

## Scope

### Files that change

| File | What changes |
|------|-------------|
| `src/tools/cargo_init.rs` | New tool |
| `src/tools/cargo_new.rs` | New tool |
| `src/tools.rs` | Register both in `tools!` macro |

### Design notes
- `cargo init` and `cargo new` are similar but have different
  directory semantics. Two tools is clearer than one tool with a
  mode flag.
- These don't need `ensure_rust_project` — they create projects.
  They use the working directory (init) or create a new one (new).

## Success Criteria

- [ ] SC1: `cargo_init { lib: true }` creates a library project in the
      working directory (name inferred from dir)
- [ ] SC2: `cargo_new { name: "my-app" }` creates a new binary
      project directory
- [ ] SC3: `edition` param sets the Rust edition
- [ ] SC4: `vcs: "none"` skips git init
- [ ] SC5: All tests pass, clippy clean, fmt clean

## Anti-Patterns (FORBIDDEN)

- NO combining init and new into one tool with a mode param
- NO requiring `set_working_directory` before `cargo_new`
