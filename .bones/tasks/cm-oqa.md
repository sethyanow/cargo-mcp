---
id: cm-oqa
title: Add cargo_audit tool
status: open
type: feature
priority: 3
phase: design
---

## Context

`cargo audit` checks dependencies against the RustSec Advisory Database
for known vulnerabilities. If the MCP server is the agent's only
interface to cargo, it should cover security basics. Agents working on
production code should be able to check for known vulnerabilities
without shell access.

`cargo-audit` is a separate binary (`cargo install cargo-audit`), not a
built-in cargo subcommand. The tool should handle the case where it's
not installed.

## Requirements

R1. A `cargo_audit` tool MUST exist that runs `cargo audit`.

R2. The tool MUST support:
    - `toolchain`, `cargo_env`, `extra_args` — standard params

R3. If `cargo-audit` is not installed, the tool MUST return a clear
    error message suggesting installation, not a cryptic spawn failure.

R4. Default output MUST be JSON (`--json`). Agents are the consumer.
    Use `extra_args` for any other flags (`--deny`, etc.).

## Scope

### Files that change

| File | What changes |
|------|-------------|
| `src/tools/cargo_audit.rs` | New tool |
| `src/tools.rs` | Register `CargoAudit` in `tools!` macro |

### Design notes
- `cargo audit` runs against `Cargo.lock`, which must exist. If
  missing, cargo audit errors — let that error propagate naturally.
- Unlike other tools, `cargo audit` is not a cargo subcommand — it's
  a standalone binary. `Command::new("cargo").arg("audit")` works
  because cargo delegates to `cargo-audit` binary on PATH.
- Consider startup probe similar to nextest (cm-rp0) to detect
  availability, but may be overkill for a less-common tool. Simpler
  to handle at call time with a clear error.

## Success Criteria

- [ ] SC1: `cargo_audit {}` runs `cargo audit --json` and returns structured JSON
- [ ] SC2: Missing `cargo-audit` binary returns helpful error message
- [ ] SC4: All tests pass, clippy clean, fmt clean

## Anti-Patterns (FORBIDDEN)

- NO auto-installing cargo-audit (agent/user decision)
- NO startup probe for cargo-audit availability (handle at call time)
- NO suppressing audit findings or filtering by severity
  (agent decides what to act on)
