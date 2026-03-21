---
id: cm-paw
title: 'Phase 1: Individual Tool Enhancements'
status: open
type: epic
priority: 1
depends_on: [cm-ie5, cm-z97, cm-f9o, cm-viy, cm-jh1]
parent: cm-1k8
---














## Context
Parent epic cm-1k8 (Cargo MCP Tool Enhancements), Phase 1. No prior phases.
Enhances four existing tools with new params and adds one new tool. All changes are independent within this phase.

## Requirements
R1 (from parent): CargoClippy `all_targets` param → `--all-targets` flag.
R2 (from parent): CargoTest `use_nextest` param → `cargo nextest run` with correct arg mapping.
R3 (from parent): CargoFmtCheck → CargoFmt rename, `check` param (default true).
R4 (from parent): New CargoDoc tool with standard params + `no_deps`, `document_private_items`.

## Success Criteria
- [x] `cargo_clippy` with `all_targets: true` includes `--all-targets` in command
- [x] `cargo_test` with `use_nextest: true` produces `cargo nextest run` command
- [x] `cargo_test` with `use_nextest: true, no_capture: true` uses `--no-capture` not `-- --nocapture`
- [x] `cargo_fmt_check` tool name gone; `cargo_fmt` tool exists
- [x] `cargo_fmt` defaults to check mode; `check: false` runs write mode
- [x] `cargo_doc` tool produces correct `cargo doc` commands
- [x] All existing tests pass
- [x] New behavior covered by tests

## Anti-Patterns
- NO separate CargoNextest or new CargoFmt struct (refactor, not duplicate)
- NO defaulting CargoFmt `check` to false (check is the safe default)

## Key Considerations
- Renaming CargoFmtCheck → CargoFmt is a breaking tool name change (cargo_fmt_check → cargo_fmt)
- Nextest arg mapping: `--nocapture` → `--no-capture`, test name stays positional
- `rustup run <toolchain> cargo nextest run` works (nextest is PATH-based)

## Acceptance Requirements
**Agent Documentation:**
- [ ] README.md: update tool list if it documents available tools
- [ ] Update examples if any reference cargo_fmt_check

**User Walkthrough Must Cover:**
- Clippy with `all_targets: true` lints test code
- Test with `use_nextest: true` runs via nextest
- Fmt in both check and write modes
- Doc tool generates documentation
- At least one error path (e.g., nextest not installed)
