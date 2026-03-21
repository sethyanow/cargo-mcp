---
id: cm-jh1
title: 'Phase 1 Acceptance: Individual Tool Enhancements'
status: closed
type: task
parent: cm-paw
---






## Context
Phase 1 sub-epic cm-paw complete — all R1-R4 implementation tasks closed (cm-ie5, cm-z97, cm-f9o, cm-viy). This acceptance task gates Phase 1 closure and Phase 2 unblocking.

## Requirements
1. Update project documentation (CLAUDE.md) to reflect new/changed tools
2. Produce user walkthrough with observable CLI commands verifying Phase 1 work

## Success Criteria
- [ ] CLAUDE.md updated with CargoDoc tool, CargoFmt rename, nextest support, all_targets param
- [ ] User walkthrough covers all 4 tool changes (clippy all_targets, test nextest, fmt rename, doc tool)
- [ ] Walkthrough presented to user in conversation
- [ ] User closes this task after reviewing

## Acceptance Requirements (from sub-epic)

**Agent Documentation:**
- Update CLAUDE.md tool list to include cargo_doc
- Update any references to cargo_fmt_check → cargo_fmt
- Document nextest support on cargo_test
- Document all_targets support on cargo_clippy

**User Walkthrough Must Cover:**
- Clippy with `all_targets: true` lints test code
- Test with `use_nextest: true` runs via nextest
- Fmt in both check and write modes
- Doc tool generates documentation

## Log

- [2026-03-21T21:47:26Z] [Seth] Acceptance verified via live MCP calls: clippy all_targets, test nextest (with no_capture + test_name), fmt check/write modes, doc with no_deps/document_private_items. All 7 calls produced correct commands. CLAUDE.md and README.md already updated in 1eaf976. No stale cargo_fmt_check references remain. 28/28 tests pass via nextest.
