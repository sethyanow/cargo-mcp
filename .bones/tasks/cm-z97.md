---
id: cm-z97
title: Add use_nextest param to CargoTest
status: open
type: task
priority: 1
parent: cm-paw
---



## Context
Phase 1 sub-epic cm-paw, parent epic cm-1k8. Second task — refactors existing CargoTest tool.
File: `src/tools/cargo_test.rs`. Current test invocation builds args as `["test", "--package", pkg, test_name, "--", "--nocapture"]`.
When nextest mode is enabled, command changes to `cargo nextest run` with different arg mapping.

**Blocked by:** cm-ie5 (closed) — established `build_args()` pattern
**Unlocks:** Remaining Phase 1 tasks (R3, R4) can proceed in parallel; Phase 1 completion

## Requirements
R2: Add `use_nextest: Option<bool>` field to `CargoTest` struct. When true:
- Command switches from `cargo test` to `cargo nextest run` (args start with `["nextest", "run"]` instead of `["test"]`)
- `no_capture` flag changes from `-- --nocapture` to `--no-capture` (direct nextest flag, no `--` separator)
- `package` and `test_name` handling stays the same (positional in both modes)

## Implementation
1. Write failing test `test_nextest_mode_produces_nextest_run` — construct CargoTest with `use_nextest: Some(true)`, call `build_args()`, assert args start with `["nextest", "run"]`. Expected RED: compile error (no field, no method).
2. Scaffolding: add `use_nextest: Option<bool>` field to CargoTest struct, add stub `pub fn build_args(&self) -> Vec<String>` that returns current `["test", ...]` logic (no nextest branching). Expected RED: assertion failure (args start with "test").
3. GREEN: implement nextest branching in `build_args()`. If `use_nextest.unwrap_or(false)`: start with `["nextest", "run"]`, use `--no-capture` for no_capture; else standard `["test"]` with `-- --nocapture`.
4. Write test `test_nextest_no_capture_flag` — `use_nextest: true`, `no_capture: true`. Assert `--no-capture` present, `--nocapture` absent, no `--` separator. Should already PASS from step 3.
5. Write test `test_standard_mode_no_capture` — `use_nextest: None`, `no_capture: true`. Assert `--` then `--nocapture` (original behavior preserved).
6. Write test `test_nextest_all_fields` — all fields set. Assert full ordering: `["nextest", "run", "--package", "foo", "bar", "--no-capture"]`.
7. Refactor `execute()` to call `build_args()`, convert `Vec<String>` to `Vec<&str>` for `create_cargo_command`. Update command label to "cargo nextest run" when in nextest mode.
8. Add `WithExamples` entry for nextest mode (non-logic change, docs escape hatch).
9. Full suite + clippy + fmt, commit, push.

## Success Criteria
- [ ] `CargoTest` has `use_nextest: Option<bool>` field
- [ ] When `use_nextest` is true, args start with `["nextest", "run"]` not `["test"]`
- [ ] When `use_nextest` is true and `no_capture` is true, args contain `--no-capture` (not `-- --nocapture`)
- [ ] When `use_nextest` is None/false, behavior unchanged (standard `cargo test` with `-- --nocapture`)
- [ ] Example added to `WithExamples`
- [ ] Tests verify generated args via `build_args()` (not struct field existence)
- [ ] `cargo test` passes

## Anti-Patterns
- NO creating a separate CargoNextest struct (refactor existing CargoTest — epic anti-pattern)
- NO validating whether nextest is installed (let cargo/nextest error naturally)
- NO tautological tests that only verify struct field values
- NO end-to-end tests that invoke cargo test/nextest

## Key Considerations
- CargoTest derives `Default` — new `Option<bool>` field defaults to `None` automatically, no need to update `..Self::default()` patterns in examples
- `create_cargo_command` handles `rustup run <toolchain> cargo` prefix — `["nextest", "run"]` works correctly because `rustup run nightly cargo nextest run` is valid
- Use `crate::tools::CargoTest` in tests (not module path — macro keeps modules private)
- In nextest mode, `test_name` is a positional filter (same as standard mode)
