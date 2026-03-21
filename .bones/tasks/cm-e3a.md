---
id: cm-e3a
title: Add extra_args to bench and run tools (conditional -- separators)
status: open
type: task
parent: cm-c1g
---

## Context
Phase 2 of cm-1k8, final implementation task. cm-tb9 (closed) established the extra_args splice-before-separator pattern on clippy/test. cm-w82 (closed) added extra_args to the 6 simple tools (no separator). This task adds extra_args to the last 2 tools — both have conditional `--` separators that require splice-before logic.

SetWorkingDirectory doesn't run cargo — extra_args doesn't apply. Flagged for user decision if R5's "all tools" is intended literally.

**Blocked by:** cm-w82 (closed — all simple tools done)
**Unlocks:** Completes cm-c1g criteria 1 and 2 (12/12 tools). After this, only acceptance task remains.

## Requirements
From parent epic R5: All tool structs MUST support `extra_args: Option<Vec<String>>` that passes arbitrary cargo-level arguments before any `--` separator.

## Design

### CargoBench separator semantics
`--` is conditional: only present when `baseline` is Some.
Current execute() ordering (lines 97-109):
1. `["bench"]`
2. optional `--package pkg`
3. optional `bench_name` (positional)
4. IF baseline: `["--", "--save-baseline", baseline]`

extra_args go between step 3 and step 4 — after all cargo-level flags, before the conditional `--`.

### CargoRun separator semantics
`--` is conditional: only present when `args` is Some and non-empty.
Current execute() ordering (lines 122-160):
1. `["run"]`
2. optional `--package pkg`
3. optional `--bin bin`
4. optional `--example example`
5. optional `--release`
6. optional `--features features`
7. optional `--all-features`
8. optional `--no-default-features`
9. IF args non-empty: `["--", arg1, arg2, ...]`

extra_args go between step 8 and step 9.

Note: CargoRun uses `#[derive(Default)]` and `..Self::default()` in WithExamples — adding extra_args field with `Option` type works with Default derive. No need to change example struct literals (they use `..Self::default()`).

### CargoBench WithExamples
Uses full struct literals (no `..Self::default()`). All 4 examples need `extra_args: None` added.

## Implementation

### Step 1: Write failing tests for CargoBench
File: `src/tests.rs`
Test 1: `bench_default_produces_minimal_args` — baseline: all fields None + `extra_args: None`, assert `["bench"]`.
Test 2: `bench_extra_args_before_separator` — `baseline: Some("main"), extra_args: Some(vec!["--features", "foo"])`, assert extra_args appear BEFORE the `--` separator. Use positional assertions like the clippy tests.
Test 3: `bench_extra_args_no_separator` — `baseline: None, extra_args: Some(vec!["--features", "foo"])`, assert extra_args appended (no `--` present).
Run — expect compile errors.

### Step 2: Add extra_args + build_args to CargoBench
File: `src/tools/cargo_bench.rs`
- Add `extra_args: Option<Vec<String>>` field after `cargo_env`
- Extract `pub fn build_args(&self) -> Vec<String>` from `execute()` matching current ordering
- Splice extra_args BEFORE the conditional `--` separator (same pattern as CargoClippy)
- Update `execute()`: `let args = self.build_args(); let args_refs: Vec<&str> = ...`
- Add `extra_args: None` to all 4 WithExamples struct literals
Run tests — bench tests should pass.

### Step 3: Write failing tests for CargoRun
File: `src/tests.rs`
Test 1: `run_default_produces_minimal_args` — baseline: `CargoRun::default()` with extra_args None, assert `["run"]`.
Test 2: `run_extra_args_before_separator` — `args: Some(vec!["--verbose"]), extra_args: Some(vec!["--release"])`, assert extra_args before `--` separator and binary args after.
Test 3: `run_extra_args_no_separator` — `args: None, extra_args: Some(vec!["--release"])`, assert extra_args appended (no `--` present).
Test 4: `run_all_fields_set_ordering` — all fields populated including extra_args, verify exact ordering matches design.
Run — expect compile errors.

### Step 4: Add extra_args + build_args to CargoRun
File: `src/tools/cargo_run.rs`
- Add `extra_args: Option<Vec<String>>` field after `cargo_env`
- Extract `pub fn build_args(&self) -> Vec<String>` from `execute()` matching current ordering
- Splice extra_args BEFORE the conditional `--` separator
- Update `execute()`: `let args = self.build_args(); let args_refs: Vec<&str> = ...`
- WithExamples uses `..Self::default()` — no changes needed (Default derive handles `extra_args: None`)
Run tests — all should pass.

### Step 5: Full test suite + clippy
Run all tests (should be ~64: 57 existing + 7 new). `cargo_check`, `cargo_clippy { all_targets: true }`.

## Success Criteria
- [ ] CargoBench has build_args() and extra_args field
- [ ] CargoRun has build_args() and extra_args field
- [ ] CargoBench splices extra_args before conditional `--` separator
- [ ] CargoRun splices extra_args before conditional `--` separator
- [ ] Both tools pass extra_args through when no `--` separator present
- [ ] No validation of extra_args contents
- [ ] All existing tests pass
- [ ] New tests verify both with-separator and without-separator cases
- [ ] Baseline tests verify build_args() with extra_args: None matches current execute() behavior

## Anti-Patterns
- NO validation or filtering of extra_args (from parent epic)
- NO default extra_args values — None/empty only
- NO changing existing behavior when extra_args is None
- NO skipping build_args() extraction — inline splice in execute() is not testable
- NO reordering args relative to current execute() — baseline tests enforce this
- NO adding extra_args to SetWorkingDirectory without user decision
