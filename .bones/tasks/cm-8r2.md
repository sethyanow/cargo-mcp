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
1. Update CLAUDE.md to document extra_args capability
2. Provide user walkthrough with CLI commands verifying Phase 2's work

## Implementation

### Deliverable 1: Agent Documentation
Update CLAUDE.md to reflect extra_args on all tools.

### Deliverable 2: User Walkthrough
CLI commands with observable outcomes covering:
- Passing `--no-default-features` via extra_args on at least one tool
- Passing `--features "foo"` via extra_args
- Passing `--lib` to cargo_test via extra_args
- Verify extra_args don't interfere with tool-specific args

## Success Criteria
- [ ] CLAUDE.md updated with extra_args documentation
- [ ] User walkthrough presented and verified
