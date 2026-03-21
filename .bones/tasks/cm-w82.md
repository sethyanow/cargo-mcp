---
id: cm-w82
title: Add extra_args to remaining simple tools (check, build, add, remove, update, clean)
status: active
type: task
parent: cm-c1g
---


## Context
Phase 2 of cm-1k8 (Cargo MCP Tool Enhancements). cm-tb9 (closed) added `extra_args` to the 4 tools with `build_args()`. This task adds `extra_args` to the 6 "simple" tools that have no `--` separator. These tools currently construct args inline in `execute()` using `&str` — they need `build_args()` extraction for testability (established pattern from Phase 1).

CargoBench and CargoRun have conditional `--` separators — they are a separate follow-up task.
SetWorkingDirectory doesn't run cargo at all — extra_args doesn't apply. Flag for user decision if R5's "all tools" is intended literally.

**Blocked by:** cm-tb9 (closed — pattern validated)
**Unlocks:** CargoBench/CargoRun task (last 2 tools), satisfies cm-c1g criteria 1 and 2 (partially — 10/12 tools done after this)

## Requirements
From parent epic R5: All tool structs MUST support `extra_args: Option<Vec<String>>` that passes arbitrary cargo-level arguments before any `--` separator.

## Design

### Common pattern for all 6 tools

None of these tools have `--` separators. The pattern is:
1. Add `extra_args: Option<Vec<String>>` field to struct
2. Extract `build_args() -> Vec<String>` from inline `execute()` args construction
3. Append `extra_args` at end of `build_args()`
4. Update `execute()` to call `build_args()` + convert via `args_refs: Vec<&str>`
5. Update `WithExamples` struct literals with `extra_args: None`

### Tool-specific notes

- **CargoCheck:** Fields: package. Simplest tool — just `["check"]` + optional `--package`.
- **CargoBuild:** Fields: package, release. `["build"]` + optional `--package` + optional `--release`.
- **CargoAdd:** Fields: dependencies (required Vec), package, dev, optional, features. Most complex arg construction of the 6. `dependencies` is `Vec<String>` not `Option`.
- **CargoRemove:** Fields: dependencies (required Vec), package, dev. Current execute() ordering: `["remove"]` + optional `--package` + optional `--dev` + dependencies. Dependencies are positional and come LAST.
- **CargoUpdate:** Fields: package, dependencies (optional Vec), dry_run. Current execute() ordering: `["update"]` + optional `--package` + optional `--dry-run` + per-dep `--package` entries. Note: `--dry-run` comes BEFORE per-dep entries.
- **CargoClean:** Fields: package. `["clean"]` + optional `--package`.

### Ordering

Group by complexity: simplest first (check, clean), then medium (build, remove), then complex (update, add). This builds confidence with trivial cases before hitting CargoAdd's arg complexity.

## Implementation

### Step 1: Write failing tests for CargoCheck
File: `src/tests.rs`
Test 1: `check_default_produces_minimal_args` — construct CargoCheck with all fields None + `extra_args: None`, call `build_args()`, assert `["check"]`. This is the BASELINE test that locks in current execute() behavior before refactoring.
Test 2: `check_extra_args_appended` — construct CargoCheck with `extra_args: Some(vec!["--all-targets"])`, call `build_args()`, assert `["check", "--all-targets"]` (no `--package` when None).
Run: `cargo_test { test_name: "check_" }` — expect compile error (no `build_args()` method, no `extra_args` field).

### Step 2: Add extra_args + build_args to CargoCheck
File: `src/tools/cargo_check.rs`
- Add `extra_args: Option<Vec<String>>` field after `cargo_env`
- Extract `pub fn build_args(&self) -> Vec<String>` from `execute()`: move the `["check"]` + `--package` logic
- Append extra_args at end of `build_args()`
- Update `execute()` to: `let args = self.build_args(); let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();`
- Update WithExamples struct literals with `extra_args: None`
Run tests — new test should pass, existing tests still pass.

### Step 3: Write failing tests for CargoClean
File: `src/tests.rs`
Test 1: `clean_default_produces_minimal_args` — baseline: all fields None + `extra_args: None`, assert `["clean"]`.
Test 2: `clean_extra_args_appended` — same pattern as check.
Run — expect compile error.

### Step 4: Add extra_args + build_args to CargoClean
File: `src/tools/cargo_clean.rs`
Same pattern as CargoCheck. Very similar structure (just `["clean"]` + optional package).
Update WithExamples. Run tests.

### Step 5: Write failing tests for CargoBuild
File: `src/tests.rs`
Test 1: `build_default_produces_minimal_args` — baseline: all fields None + `extra_args: None`, assert `["build"]`.
Test 2: `build_extra_args_appended` — construct with `release: Some(true), extra_args: Some(vec!["--jobs", "4"])`, assert `["build", "--release", "--jobs", "4"]`.
Run — expect compile error.

### Step 6: Add extra_args + build_args to CargoBuild
File: `src/tools/cargo_build.rs`
Extract `build_args()`: `["build"]` + optional `--package` + optional `--release` + extra_args.
Update execute(). Update WithExamples. Run tests.

### Step 7: Write failing tests for CargoRemove
File: `src/tests.rs`
Test 1: `remove_default_produces_minimal_args` — baseline: `dependencies: vec!["serde"]`, all optional fields None + `extra_args: None`, assert `["remove", "serde"]`.
Test 2: `remove_extra_args_appended` — construct with `dependencies: vec!["serde"], extra_args: Some(vec!["--dry-run"])`, assert `["remove", "serde", "--dry-run"]`.
Run — expect compile error.

### Step 8: Add extra_args + build_args to CargoRemove
File: `src/tools/cargo_remove.rs`
Extract `build_args()` matching current execute() ordering: `["remove"]` + optional `--package` + optional `--dev` + dependencies + extra_args. Dependencies are positional and come AFTER flags (match current code). `dependencies` is `Vec<String>` (required), not `Option`.
Update execute(). Update WithExamples. Run tests.

### Step 9: Write failing tests for CargoUpdate
File: `src/tests.rs`
Test 1: `update_default_produces_minimal_args` — baseline: all fields None + `extra_args: None`, assert `["update"]`.
Test 2: `update_extra_args_appended` — construct with `dry_run: Some(true), extra_args: Some(vec!["--recursive"])`, assert `["update", "--dry-run", "--recursive"]`.
Run — expect compile error.

### Step 10: Add extra_args + build_args to CargoUpdate
File: `src/tools/cargo_update.rs`
Extract `build_args()` matching current execute() ordering: `["update"]` + optional `--package pkg` + optional `--dry-run` + per-dep `--package dep` entries + extra_args. Note: `--dry-run` comes BEFORE per-dep entries (match current code lines 97-112).
Update execute(). Update WithExamples. Run tests.

### Step 11: Write failing tests for CargoAdd
File: `src/tests.rs`
Test 1: `add_default_produces_minimal_args` — baseline: `dependencies: vec!["serde"]`, all optional fields None + `extra_args: None`, assert `["add", "serde"]`.
Test 2: `add_extra_args_appended` — construct with `dependencies: vec!["serde"], extra_args: Some(vec!["--offline"])`, assert `["add", "serde", "--offline"]`.
Run — expect compile error.

### Step 12: Add extra_args + build_args to CargoAdd
File: `src/tools/cargo_add.rs`
Extract `build_args()` matching current execute() ordering: `["add"]` + optional `--package` + optional `--dev` + optional `--optional` + optional `--features features_str` + dependencies + extra_args. Dependencies are positional and come AFTER all flags (match current code lines 118-143). `features_str = features.join(",")` — the local variable lifetime is simpler in build_args() since all args are owned Strings.
Update execute(). Update WithExamples. Run tests.

### Step 13: Full test suite + clippy
Run all tests (should be ~50: 38 existing + 12 new = 6 baseline + 6 extra_args). `cargo_check`, `cargo_clippy { all_targets: true }`.

## Success Criteria
- [ ] CargoCheck has build_args() and extra_args field
- [ ] CargoBuild has build_args() and extra_args field
- [ ] CargoAdd has build_args() and extra_args field
- [ ] CargoRemove has build_args() and extra_args field
- [ ] CargoUpdate has build_args() and extra_args field
- [ ] CargoClean has build_args() and extra_args field
- [ ] All 6 tools append extra_args after tool flags
- [ ] No validation of extra_args contents
- [ ] All existing tests pass
- [ ] New tests verify extra_args for each tool
- [ ] Each tool has a baseline test verifying build_args() with extra_args: None matches current execute() behavior

## Key Considerations

- **Arg ordering must match current execute() exactly.** The build_args() extraction changes from `&str` to `String`, but the ORDER of args must not change. Each tool's execute() has a specific ordering — read it and replicate exactly. Ordering differences violate anti-pattern "NO changing existing behavior when extra_args is None." The baseline tests catch this.
- **CargoAdd features let-chain.** Current code uses `let features_str;` as a local then `if let Some(ref features) = self.features && !features.is_empty()`. In build_args() with owned Strings, this simplifies — `features.join(",")` is pushed as an owned String. No lifetime issues.
- **CargoAdd/CargoRemove empty deps validation.** Both tools have `if self.dependencies.is_empty() { return Err(...) }` in execute(). This validation stays in execute(), NOT in build_args(). build_args() is a pure arg builder.

## Anti-Patterns
- NO validation or filtering of extra_args (from parent epic)
- NO default extra_args values — None/empty only
- NO changing existing behavior when extra_args is None
- NO skipping build_args() extraction — inline splice in execute() is not testable
- NO adding extra_args to SetWorkingDirectory (not a cargo command) without user decision
- NO reordering args in build_args() relative to current execute() — baseline tests enforce this. If a baseline test fails, the build_args() ordering is wrong, not the test.
