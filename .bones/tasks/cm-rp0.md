---
id: cm-rp0
title: Auto-detect nextest and use by default
status: open
type: task
priority: 2
phase: design
---

## Context

`cargo_test` defaults to `cargo test` unless the agent explicitly passes
`use_nextest: true`. Most Rust developers who have nextest installed
prefer it — better output, parallel execution, per-test timeouts. Agents
shouldn't need to know or remember to request it.

## Requirements

R1. When `use_nextest` is `None` (not explicitly set), probe for nextest
    availability and use it if present. Fall back to `cargo test` if not.

R2. Explicit `use_nextest: true` still forces nextest (errors if missing).
    Explicit `use_nextest: false` still forces `cargo test`.

R3. The probe MUST run once at startup in `CargoTools::new()` and store
    the result as a `bool`. No lazy init, no per-call probing.

## Scope

### Files that change

| File | What changes |
|------|-------------|
| `src/state.rs` | Add `nextest_available: bool` field, probe in `CargoTools::new()` |
| `src/tools/cargo_test.rs` | `execute()` resolves `use_nextest: None` via `state.nextest_available` before `build_args()` |

### Files that DON'T change
- `build_args()` — already handles the nextest/standard split based on
  the resolved boolean. No logic change needed there.
- Other tool files — nextest only applies to `cargo_test`

### Design notes
- Probe approach: `Command::new("cargo").args(["nextest", "--version"])`
  in `CargoTools::new()`. Exit 0 = available. Store as `bool`.
- `build_args()` stays pure (takes resolved bool). Resolution happens in
  `execute()` where state access is available.

## Success Criteria

- [ ] SC1: `cargo_test {}` (no params) uses nextest when it's installed
- [ ] SC2: `cargo_test {}` falls back to `cargo test` when nextest is absent
- [ ] SC3: `cargo_test { use_nextest: false }` forces `cargo test` even if nextest is installed
- [ ] SC4: `cargo_test { use_nextest: true }` forces nextest (errors if missing)
- [ ] SC5: Probe runs once at startup in `CargoTools::new()`
- [ ] SC6: All tests pass, clippy clean, fmt clean

## Anti-Patterns (FORBIDDEN)

- NO lazy init or per-call probing (startup only)
- NO changing `build_args()` signature or purity (resolution in `execute()`)
- NO env var for this — auto-detect is the whole point
- NO swallowing nextest probe errors silently when `use_nextest: true` (explicit request must fail loudly)
