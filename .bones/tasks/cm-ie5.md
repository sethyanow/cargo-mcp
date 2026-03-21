---
id: cm-ie5
title: Add all_targets param to CargoClippy
status: open
type: task
priority: 1
parent: cm-paw
---



## Context
Phase 1 sub-epic cm-paw, parent epic cm-1k8. First task — smallest isolated change.
File: `src/tools/cargo_clippy.rs`. Current clippy invocation builds args as `["clippy", "--package", pkg, "--", "-D", "warnings"]`. The `--all-targets` flag needs to go before the `--` separator.

## Requirements
R1: Add `all_targets: Option<bool>` field to `CargoClippy` struct. When true, insert `"--all-targets"` into the args before the `-- -D warnings` separator.

## Implementation
1. Add `all_targets: Option<bool>` field to `CargoClippy` struct (with standard serde/arg derives)
2. In `execute()`, after the `--package` block and before `args.extend_from_slice(&["--", "-D", "warnings"])`, conditionally push `"--all-targets"`
3. Add an example to `WithExamples` showing `all_targets: Some(true)`
4. Add a test verifying the flag appears in the generated command

## Success Criteria
- [ ] `CargoClippy` has `all_targets: Option<bool>` field
- [ ] When `all_targets` is true, `--all-targets` appears in the command before `--`
- [ ] When `all_targets` is None/false, behavior unchanged
- [ ] Example added to `WithExamples`
- [ ] Test covers the new param
- [ ] `cargo test` passes

## Anti-Patterns
- NO making `all_targets` default to true (would change existing behavior for all callers)
- NO adding `all_targets` to other tools in this task (use extra_args in Phase 2 for that)
