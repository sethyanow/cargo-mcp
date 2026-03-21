---
id: cm-tb9
title: Add extra_args to Phase 1 tools (clippy, test, fmt, doc)
status: open
type: task
parent: cm-c1g
---

## Context
Phase 2 of cm-1k8 (Cargo MCP Tool Enhancements). Phase 1 (cm-paw) is closed — all 4 tools have `build_args()` methods. This task adds the `extra_args` field to the 4 tools that already have `build_args()`, validating the splice pattern before applying it to the remaining 8 tools.

**Blocked by:** None (Phase 1 complete)
**Unlocks:** cm-??? (remaining 8 tools get extra_args), satisfies 3/5 cm-c1g success criteria

## Requirements
From parent epic R5: All tool structs MUST support `extra_args: Option<Vec<String>>` that passes arbitrary cargo-level arguments before any `--` separator.

## Design

### Splice semantics by tool

- **CargoClippy:** Always has `--` at end (line 117 of cargo_clippy.rs). Splice extra_args before the `--` separator.
- **CargoTest (standard):** `--` only present when `no_capture` is true. Splice extra_args before `--` if present, else append.
- **CargoTest (nextest):** No `--` separator. Append extra_args after tool-specific flags.
- **CargoFmt:** No `--` separator. Append extra_args after `--check` (if present).
- **CargoDoc:** No `--` separator. Append extra_args after all flags.

### Field definition (same on all 4 structs)

```rust
extra_args: Option<Vec<String>>
```

With standard serde/schemars derives. Doc comment: "Additional cargo arguments passed before any `--` separator". `skip_serializing_if = "Option::is_none"`.

### Splice approach in `build_args()`

For tools with `--` separator: build cargo-level args, splice extra_args, then add `--` and tool-specific args.
For tools without: build all args, then append extra_args at end.

No validation of extra_args contents (anti-pattern from epic: cargo handles invalid args).

## Implementation

### Step 1: Write failing tests for CargoClippy extra_args
File: `src/tests.rs`
Test: `clippy_extra_args_before_separator` — construct CargoClippy with `extra_args: Some(vec!["--no-default-features"])`, call `build_args()`, assert `--no-default-features` appears BEFORE the `--` separator position.
Test: `clippy_no_extra_args_unchanged` — construct with `extra_args: None`, assert output matches current behavior exactly.
Run: `cargo_test { test_name: "clippy_extra_args" }` — expect compile error (field doesn't exist).

### Step 2: Add extra_args field to CargoClippy struct
File: `src/tools/cargo_clippy.rs`
Add field after `all_targets`. Update `build_args()` to splice `extra_args` before the `--` separator (line 117). The current code builds cargo flags, then unconditionally pushes `["--", "-D", "warnings"]`. Restructure: build cargo flags → splice extra_args → push `--` separator and lint args.
Update `WithExamples` — existing examples set `extra_args: None`.
Run tests — new tests should pass, existing clippy tests still pass.

### Step 3: Write failing tests for CargoTest extra_args
File: `src/tests.rs`
Test: `test_standard_extra_args_before_separator` — CargoTest with `no_capture: true, extra_args: Some(vec!["--lib"])`. Assert `--lib` appears before `--` separator.
Test: `test_standard_extra_args_no_separator` — CargoTest default with `extra_args: Some(vec!["--lib"])`. Assert `--lib` appears, no `--` separator present.
Test: `test_nextest_extra_args` — CargoTest with `use_nextest: true, extra_args: Some(vec!["--lib"])`. Assert `--lib` appears after `"run"`.
Run — expect compile error.

### Step 4: Add extra_args to CargoTest struct
File: `src/tools/cargo_test.rs`
Add field. Update `build_args()`: splice extra_args after tool-specific flags but before any `--` push. For standard mode with no_capture, the `--` is pushed conditionally — extra_args go before that conditional block.
Update WithExamples. Run tests.

### Step 5: Write failing tests for CargoFmt extra_args
File: `src/tests.rs`
Test: `fmt_extra_args_appended` — CargoFmt with `check: None, extra_args: Some(vec!["--config-path", "custom.toml"])`. Assert extra_args appear after `--check`.
Run — expect compile error.

### Step 6: Add extra_args to CargoFmt struct
File: `src/tools/cargo_fmt.rs`
Add field. Update `build_args()`: append extra_args at end (no `--` separator in fmt). Update WithExamples. Run tests.

### Step 7: Write failing tests for CargoDoc extra_args
File: `src/tests.rs`
Test: `doc_extra_args_appended` — CargoDoc with `extra_args: Some(vec!["--all-features"])`. Assert `--all-features` appears in args.
Run — expect compile error.

### Step 8: Add extra_args to CargoDoc struct
File: `src/tools/cargo_doc.rs`
Add field. Update `build_args()`: append extra_args at end. Update WithExamples. Run tests.

### Step 9: Full test suite
Run all 28+ tests. Verify no regressions. `cargo_check`, `cargo_clippy { all_targets: true }`.

## Success Criteria
- [ ] CargoClippy extra_args spliced before `--` separator
- [ ] CargoTest extra_args spliced before `--` in standard mode, appended in nextest mode
- [ ] CargoFmt extra_args appended after tool flags
- [ ] CargoDoc extra_args appended after tool flags
- [ ] No validation of extra_args contents
- [ ] All existing tests pass
- [ ] New tests verify extra_args placement for each tool

## Anti-Patterns
- NO validation or filtering of extra_args (from parent epic)
- NO default extra_args values — None/empty only
- NO changing existing build_args() behavior when extra_args is None
