---
id: cm-f9o
title: Rename CargoFmtCheck to CargoFmt with check param
status: closed
type: task
parent: cm-paw
---




## Context
Phase 1 sub-epic cm-paw, parent epic cm-1k8. Third task — refactors existing CargoFmtCheck tool.
File: `src/tools/cargo_fmt_check.rs` → rename to `src/tools/cargo_fmt.rs`. Struct `CargoFmtCheck` → `CargoFmt`.
Current tool only runs `cargo fmt --check`. New `check` param (default true) controls whether `--check` is passed.

**Blocked by:** cm-z97 (closed) — no code dependency, but sequential per epic flow
**Unlocks:** R4 (CargoDoc) can proceed; Phase 1 completion after R4

## Requirements
R3: Rename `CargoFmtCheck` struct to `CargoFmt`. Rename file from `cargo_fmt_check.rs` to `cargo_fmt.rs`. Change MCP tool name from `cargo_fmt_check` to `cargo_fmt`. Add `check: Option<bool>` field (default true). When check is true (or None), pass `--check` to `cargo fmt`. When check is false, omit `--check` (write mode).

## Implementation
1. Write failing test `test_fmt_check_mode_produces_check_flag` — construct `CargoFmt` with `check: None` (default), call `build_args()`, assert args contain `["fmt", "--check"]`. Expected RED: compile error (no `CargoFmt` struct, no `build_args`).
2. Rename file: `git mv src/tools/cargo_fmt_check.rs src/tools/cargo_fmt.rs`. Rename struct `CargoFmtCheck` → `CargoFmt`. Update `#[serde(rename = "cargo_fmt")]`. Update `tools!` macro entry in `src/tools.rs`: `(CargoFmt, cargo_fmt, "cargo_fmt")`. Add `#[derive(Default)]`. Add `check: Option<bool>` field. Add stub `build_args()` returning `["fmt", "--check"]` (current behavior). Expected RED: assertion should now pass — this step is scaffolding.
3. Verify step 1 test passes GREEN (scaffolding produced correct default behavior).
4. Write test `test_fmt_write_mode_no_check_flag` — `check: Some(false)`, assert args are `["fmt"]` (no `--check`). Expected RED: stub always returns `--check`.
5. GREEN: implement check branching in `build_args()`. If `self.check.unwrap_or(true)`: push `--check`; else omit it.
6. Write test `test_fmt_explicit_true_check_flag` — `check: Some(true)`, assert args contain `--check`. Should already PASS.
7. Write test `test_fmt_default_produces_minimal_args` — default `CargoFmt`, assert `["fmt", "--check"]`. Should already PASS.
8. Refactor `execute()` to call `build_args()`, convert to `Vec<&str>`. Update command label: `"cargo fmt --check"` when check mode, `"cargo fmt"` when write mode.
9. Update `WithExamples`: rename struct references, update descriptions, add example for write mode (`check: Some(false)`).
10. Update doc comment on struct from "Check if code is properly formatted without modifying files" to "Run cargo fmt to check or fix code formatting".
11. Full suite + clippy + fmt, commit, push.

## Success Criteria
- [ ] `CargoFmtCheck` struct no longer exists; `CargoFmt` struct exists
- [ ] MCP tool name is `cargo_fmt` (not `cargo_fmt_check`)
- [ ] File is `src/tools/cargo_fmt.rs` (not `cargo_fmt_check.rs`)
- [ ] Default behavior (check: None) passes `--check` to cargo fmt
- [ ] `check: Some(true)` passes `--check`
- [ ] `check: Some(false)` omits `--check` (write mode)
- [ ] `build_args()` method exists and is tested
- [ ] `cargo test` passes
- [ ] Examples updated to reference `CargoFmt`

## Anti-Patterns
- NO creating a new CargoFmt struct alongside CargoFmtCheck (rename in-place — epic anti-pattern)
- NO defaulting check to false (check is the safe default — sub-epic anti-pattern)
- NO tautological tests that only verify struct field values
- NO end-to-end tests that invoke cargo fmt

## Key Considerations
- `tools!` macro entry must change from `(CargoFmtCheck, cargo_fmt_check, "cargo_fmt_check")` to `(CargoFmt, cargo_fmt, "cargo_fmt")` — all three parts change
- `CargoFmtCheck` does NOT derive `Default` currently — need to add it for `..CargoFmt::default()` in tests and examples
- No existing tests reference `CargoFmtCheck` — only `tools.rs` and the tool file itself (7 references total, 2 files)
- This is a breaking MCP tool name change: clients using `cargo_fmt_check` will need to update to `cargo_fmt`
- `git mv` preserves file history; manual rename + git add does not

## Log

- [2026-03-21T20:07:42Z] [Seth] Debrief: Clean rename + bool param. Ownership issue in execute() resolved by reordering build_args() before toolchain consumption. No workarounds. Reflections: Skeleton was accurate, all 11 steps matched. No surprises. build_args pattern now established across 3 tools. No user corrections needed. Scoped cm-viy (CargoDoc R4) as next task.
