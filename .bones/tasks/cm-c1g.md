---
id: cm-c1g
title: 'Phase 2: Extra Args Passthrough'
status: open
type: epic
priority: 2
depends_on: [cm-paw, cm-tb9, cm-w82, cm-e3a]
parent: cm-1k8
---










## Context
Parent epic cm-1k8 (Cargo MCP Tool Enhancements), Phase 2. Depends on Phase 1 (cm-paw).
After Phase 1, all tools are in final form. This phase adds `extra_args` to every tool struct.

## Requirements
R5 (from parent): All tool structs MUST support `extra_args: Option<Vec<String>>` that passes arbitrary cargo-level arguments before any `--` separator.

## Success Criteria
- [ ] Every tool struct has `extra_args: Option<Vec<String>>` field
- [ ] Extra args are spliced into command before any `--` separator
- [x] Tests verify extra_args appear in generated commands for at least clippy, test, fmt, and doc
- [x] No validation of extra_args contents (cargo handles errors)
- [x] All existing tests pass

## Anti-Patterns
- NO validating or filtering extra_args (anti-pattern from parent)
- NO default extra_args values (empty/None only)
- NO inserting extra_args after `--` separator (these are cargo-level args)
- NO adding extra_args to only some tools (R5 says all)

## Key Considerations
- Each tool has different `--` separator semantics (clippy: `-- -D warnings`, test: `-- --nocapture`, bench: `-- --save-baseline`). Extra args go before all of these.
- Tools without `--` separators (check, build, clean, add, remove, update, run, set_working_directory) just append extra_args to the args vec.

## Acceptance Requirements
**Agent Documentation:**
- [ ] README.md: document extra_args capability

**User Walkthrough Must Cover:**
- Passing `--no-default-features` via extra_args on at least one tool
- Passing `--features "foo"` via extra_args
- Passing `--lib` to cargo_test via extra_args
- Verify extra_args don't interfere with tool-specific args
