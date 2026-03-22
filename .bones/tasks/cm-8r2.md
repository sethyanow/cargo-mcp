---
id: cm-8r2
title: 'Phase 2 Acceptance: Extra Args Passthrough'
status: open
type: task
parent: cm-c1g
---







## Context
Acceptance gate for sub-epic cm-c1g (Phase 2: Extra Args Passthrough). All implementation tasks are closed (cm-tb9, cm-w82, cm-e3a). All 5 sub-epic success criteria are checked. This task verifies the phase is complete and updates documentation.

## Requirements
1. Update README.md to document extra_args capability (per sub-epic cm-c1g acceptance requirements)
2. Update CLAUDE.md tool pattern section to reflect extra_args as a standard field
3. Provide user walkthrough with MCP tool calls verifying Phase 2's work

## Implementation

### Deliverable 1: Agent Documentation

**README.md** (user-facing):
- Update line 63 ("Every tool accepts `toolchain` ... and `cargo_env`...") to include `extra_args`
- Add a brief description of what extra_args does (passes arbitrary cargo-level arguments before any `--` separator)

**CLAUDE.md** (developer-facing):
- Update the "Tool pattern" section to mention `extra_args: Option<Vec<String>>` as a standard field on all tool structs
- Update the "Build & Test Commands" section header comment to mention extra_args capability

### Deliverable 2: User Walkthrough
MCP tool calls with observable outcomes covering all 4 sub-epic acceptance items:
1. Passing `--no-default-features` via extra_args on at least one tool
2. Passing `--features "foo"` via extra_args
3. Passing `--lib` to cargo_test via extra_args
4. Verify extra_args don't interfere with tool-specific args (e.g., clippy with all_targets + extra_args)

## Anti-Patterns
- NO documenting extra_args without verifying the test suite passes first
- NO presenting walkthrough commands without actually running them to verify output
- NO updating only one of README.md/CLAUDE.md — both must be updated

## Success Criteria
- [x] README.md updated: extra_args documented alongside toolchain and cargo_env
- [x] CLAUDE.md updated: tool pattern section reflects extra_args as standard field
- [x] Full test suite passes (70 tests, verified before presenting walkthrough)
- [x] User walkthrough covers all 4 acceptance requirement items from cm-c1g
- [x] User walkthrough presented with actual MCP tool call outputs

## Log

- [2026-03-22T00:27:56Z] [Seth] Debrief: Docs updated (README.md + CLAUDE.md), walkthrough verified all 4 acceptance items via live MCP tool calls. MCP schema caching caused initial walkthrough failure — client needed reconnection after recompilation. SRE caught README.md vs CLAUDE.md discrepancy in original skeleton. Added tools_list_exposes_extra_args integration test (71 total). Reflections: MCP client schema caching is a class of end-to-end bug invisible to unit/integration tests. Saved reference memory for future sessions.
