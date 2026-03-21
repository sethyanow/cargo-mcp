---
id: cm-ie5
title: Add all_targets param to CargoClippy
status: active
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
2. In `execute()`, after the `--fix` block (line 94-96) and before `args.extend_from_slice(&["--", "-D", "warnings"])` (line 99), conditionally push `"--all-targets"` when `self.all_targets.unwrap_or(false)`
3. Add an example to `WithExamples` showing `all_targets: Some(true)`
4. Add test(s) in `src/tests.rs` that verify command arg generation. Testing approach: build the args vector the same way `execute()` does, pass to `create_cargo_command()`, then inspect the `Command` via `cmd.get_program()` and `cmd.get_args()` to verify `--all-targets` is present and positioned before `--`. Test both `Some(true)` and `None`/default cases.

## Success Criteria
- [ ] `CargoClippy` has `all_targets: Option<bool>` field
- [ ] When `all_targets` is true, `--all-targets` appears in the command before `--`
- [ ] When `all_targets` is None/false, behavior unchanged
- [ ] Example added to `WithExamples`
- [ ] Test verifies `--all-targets` appears in `Command::get_args()` before `--` (not just struct field existence)
- [ ] `cargo test` passes

## Anti-Patterns
- NO making `all_targets` default to true (would change existing behavior for all callers)
- NO adding `all_targets` to other tools in this task (use extra_args in Phase 2 for that)
- NO tautological tests that only verify the struct field value (e.g., `assert_eq!(clippy.all_targets, Some(true))`) — tests must verify the generated Command args
- NO end-to-end tests that actually invoke `cargo clippy` — test the arg building, not cargo execution

## Key Considerations
- `Command::get_args()` returns an iterator of `OsStr` — collect to `Vec<&OsStr>` or convert to strings for assertions
- The `--all-targets` flag must appear BEFORE the `--` separator in the args. Verify position, not just presence.
- The `fix` field (line 94-96) follows the same `unwrap_or(false)` pattern — use it as a model for `all_targets`

### Adversarial Planning Notes
- Most failure categories (input hostility, encoding, temporal, state, resource) don't apply — this is one boolean field with one conditional string push, no persistent state, no concurrency, no dynamic content.
- **`--all-targets` + `--fix` interaction:** When both flags are set, clippy fixes apply to test/example/bench code too. This is valid cargo behavior and both params are opt-in — no mitigation needed, but worth noting in case of future user questions.
