---
id: cm-1k8
title: Cargo MCP Tool Enhancements
status: open
type: epic
priority: 1
depends_on: [cm-paw, cm-c1g]
---





## Requirements

R1. CargoClippy MUST support an `all_targets` parameter that passes `--all-targets` to the underlying cargo clippy invocation, enabling linting of tests, examples, and benchmarks.

R2. CargoTest MUST support a `use_nextest` parameter that switches the underlying command from `cargo test` to `cargo nextest run`, with correct argument mapping (test name filtering, no-capture flag).

R3. CargoFmtCheck MUST be refactored to CargoFmt — renamed struct, file, and MCP tool name (`cargo_fmt`). MUST support a `check` parameter (default true) that controls whether `--check` is passed. When check is false, runs write mode (modifies files).

R4. A new CargoDoc tool MUST exist with `package`, `toolchain`, `cargo_env`, `no_deps` (default true), and `document_private_items` parameters. Runs `cargo doc`.

R5. All tool structs MUST support an `extra_args` parameter (`Vec<String>`) that passes arbitrary cargo-level arguments (before any `--` separator). Covers `--no-default-features`, `--features`, `--lib`, `--all-targets` on tools without dedicated params, etc.

## Success Criteria

- [x] `cargo_clippy` with `all_targets: true` produces a command containing `--all-targets`
- [x] `cargo_test` with `use_nextest: true` produces a command starting with `cargo nextest run`
- [x] `cargo_test` with `use_nextest: true` and `no_capture: true` uses `--no-capture` (not `-- --nocapture`)
- [x] MCP tool `cargo_fmt_check` no longer exists; `cargo_fmt` exists with `check` param
- [x] `cargo_fmt` with default params passes `--check`; with `check: false` omits it
- [ ] `cargo_doc` tool exists and produces correct `cargo doc` commands
- [ ] All tools accept `extra_args` and splice them before any `--` separator
- [ ] All existing tests pass
- [ ] New behavior covered by tests

## Anti-Patterns (FORBIDDEN)

- NO creating separate CargoNextest or CargoFmt tool structs (R2/R3 are refactors of existing tools, not new tools — DRY/YAGNI)
- NO conditional compilation or feature flags for nextest support (it's a runtime param, not a build-time choice)
- NO validation of extra_args contents (the user knows what they're passing; cargo will error on invalid args)
- NO default `extra_args` values (empty/None is the default — never inject args the user didn't request)

## Approach

Refactor existing tools in-place where possible (CargoTest for nextest, CargoFmtCheck → CargoFmt), add one new tool (CargoDoc), then add the cross-cutting `extra_args` param to all tools. The codebase has a uniform pattern (struct → WithExamples → Tool impl → create_cargo_command → execute_cargo_command) that makes all changes mechanical.

Phase 1 handles individual tool changes (R1-R4). Phase 2 adds the cross-cutting extra_args (R5) after tools are in final form.

## Architecture

All tools live in `src/tools/`. Each is a struct with derive macros registered via the `tools!` macro in `src/tools.rs`. Commands are built via `create_cargo_command` in `cargo_utils.rs` and executed via `execute_cargo_command`.

Key files:
- `src/tools/cargo_clippy.rs` — R1 (add param)
- `src/tools/cargo_test.rs` — R2 (add nextest mode)
- `src/tools/cargo_fmt_check.rs` → `src/tools/cargo_fmt.rs` — R3 (rename + add check param)
- `src/tools/cargo_doc.rs` — R4 (new file)
- `src/tools.rs` — registration changes for R3 rename and R4 addition
- All `src/tools/cargo_*.rs` files — R5 (add extra_args)

## Phases

### Phase 1: Individual Tool Enhancements
**Scope:** R1, R2, R3, R4
**Gate:**
- `cargo test -- tools_doesnt_panic` → passes (tool registration works after refactors)
- `cargo test` → all tests pass
- Inspect generated commands in test output for:
  - clippy `--all-targets` presence
  - nextest `cargo nextest run` base command
  - fmt `--check` toggling
  - doc `cargo doc --no-deps`

### Phase 2: Extra Args Passthrough
**Scope:** R5
**Gate:**
- `cargo test` → all tests pass
- Tests verify extra_args appear in generated commands before `--` separator for at least clippy, test, fmt, and doc tools

## Agent Failure Mode Catalog

### Phase 1
| Shortcut | Rationalization | Pre-block |
|----------|----------------|-----------|
| Create new CargoNextest struct instead of refactoring | "Cleaner separation of concerns" | Anti-pattern explicitly forbids it; R2 says refactor |
| Create new CargoFmt struct alongside CargoFmtCheck | "Backward compatibility" | Anti-pattern forbids it; R3 says rename, not add |
| Skip nextest arg mapping tests | "It's just string building" | Success criteria require command verification |
| Default `check` to false on CargoFmt | "Write mode is the primary use case" | R3 specifies default true — check mode is safer default |

### Phase 2
| Shortcut | Rationalization | Pre-block |
|----------|----------------|-----------|
| Only add extra_args to "important" tools | "Some tools don't need it" | R5 says ALL tool structs |
| Insert extra_args after `--` separator | "That's where extra flags go" | R5 specifies before `--` (cargo-level args) |
| Validate or filter extra_args | "Prevent invalid commands" | Anti-pattern: no validation, cargo handles errors |

## Seam Contracts

### Phase 1 → Phase 2
**Delivers:** All tools in final form (clippy with all_targets, test with nextest, CargoFmt renamed, CargoDoc added). Each tool's arg-building logic has a clear insertion point for cargo-level args.
**Assumes:** Phase 2 can add `extra_args` field to each struct and splice into args vec before any `--` separator.
**If wrong:** If a tool's arg building doesn't have a clear pre-`--` insertion point, Phase 2 needs to refactor that tool's execute method — minor rework, contained to one file per tool.

## Design Rationale

### Problem
cargo-mcp tools lack several capabilities that force users to shell out: clippy can't lint test targets, no nextest support, no write-mode formatting, no doc checking, and no way to pass arbitrary cargo flags.

### Research Findings
**Codebase:** All 12 tools in `src/tools/` follow identical pattern — struct with derives, WithExamples, Tool impl calling create_cargo_command/execute_cargo_command in cargo_utils.rs. Changes are mechanical.
**External:** `cargo nextest run` accepts `--package`, positional test name filter, and `--no-capture` as top-level flags. `rustup run <toolchain> cargo nextest run` works (nextest is PATH-based, not toolchain-bound).

### Approaches Considered

#### 1. Refactor existing tools + cross-cutting extra_args (selected)
**Chosen because:** DRY — reuses existing structs, avoids duplicating boilerplate. YAGNI — one tool per concept, params control behavior. Matches user's explicit preference.

#### 2. New separate tools for each variant (CargoNextest, CargoFmt alongside CargoFmtCheck)
**Why explored:** Simpler per-tool logic, no conditional branches in execute().
**REJECTED BECAUSE:** Duplicates struct boilerplate (package, toolchain, cargo_env fields), creates multiple tools for the same concept, user explicitly rejected this approach.
**DO NOT REVISIT UNLESS:** Tool impl trait requires fundamentally different signatures per variant.

### Scope Boundaries
**In scope:** The 5 features as specified (R1-R5).
**Out of scope:**
- `--all-targets` on tools other than clippy (YAGNI — use extra_args if needed)
- Nextest configuration file support (cargo-nextest handles its own config)
- cargo doc --open (MCP server has no display)

### Open Questions
- None — all design decisions resolved during brainstorming.

## Design Discovery

### Key Decisions Made
| Question | Answer | Implication |
|----------|--------|-------------|
| Nextest: new tool or refactor? | Refactor CargoTest | R2 adds `use_nextest` param, no new struct |
| Fmt: new tool or refactor? | Refactor CargoFmtCheck → CargoFmt | R3 renames struct/file/tool, adds `check` param |
| Fmt: mode param or rename? | Rename to CargoFmt | Breaking change on tool name (cargo_fmt_check → cargo_fmt), acceptable for own MCP server |

### Dead-End Paths
None — straightforward enhancement, no dead ends encountered.

### Open Concerns
- `cargo_fmt_check` → `cargo_fmt` is a breaking tool name change. Any existing MCP client configs referencing `cargo_fmt_check` will break. Acceptable per user decision.
