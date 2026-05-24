---
id: cm-7jg
title: Rename cargo-mcp to stevedore
status: open
type: task
priority: 1
phase: design
---

## Context

Renaming the project from `cargo-mcp` (upstream name) to `stevedore` for
publication as an independent crate. Upstream (jbr/cargo-mcp) is abandoned.
Name chosen: "stevedore" — the person who loads and unloads cargo.

## Requirements

R1. Package and binary name MUST change from `cargo-mcp` to `stevedore`
    across all build artifacts and install paths.

R2. All documentation MUST reference `stevedore`, not `cargo-mcp`.
    MCP client config examples MUST show `"command": "stevedore"`.

R3. Cargo.toml `authors` MUST list only the current maintainer (Seth).
    `repository` and `documentation` MUST point to the fork. Description
    MUST be updated. Original author credited in README, not Cargo.toml.

R4. Session file path (`cargo-mcp.json`) MUST rename to `stevedore.json`
    to avoid collisions if someone has both installed.

R5. The `server_info!()` macro in main.rs reads from Cargo.toml — verify
    it picks up the new name automatically (no code change expected).

## Scope

### Files that change

| File | What changes |
|------|-------------|
| `Cargo.toml` | `name`, `description`, `repository`, `documentation`, `authors`, `keywords` |
| `README.md` | All ~10 `cargo-mcp` references, install command, MCP config examples, repo URLs |
| `CLAUDE.md` | ~2 references to `cargo-mcp` in project description |
| `src/state.rs` | Session file path: `cargo-mcp.json` → `stevedore.json` |
| `src/tools/set_working_directory.rs` | Comment mentioning `cargo-mcp` |
| `src/main.rs` | INSTRUCTIONS string — update if branding needed (currently generic) |

### Files that DON'T change
- `src/tools/cargo_*.rs` — tool names stay `cargo_*` (they wrap cargo commands, not the server)
- `.bones/` — historical records, leave as-is
- Test files — no `cargo-mcp` references in test assertions

### Out of scope
- GitHub repo rename (manual, separate from code)
- crates.io publishing (separate step after rename merges)
- Changing tool names from `cargo_*` (those describe the cargo subcommands they wrap)

## Success Criteria

- [ ] SC1: `cargo install --path .` produces a `stevedore` binary
- [ ] SC2: `stevedore serve` starts the MCP server
- [ ] SC3: README contains zero references to `cargo-mcp` as the project name
      (references to upstream for attribution are fine)
- [ ] SC4: MCP config examples use `"command": "stevedore"`
- [ ] SC5: Session file writes to `stevedore.json`, not `cargo-mcp.json`
- [ ] SC6: All tests pass, clippy clean, fmt clean
- [ ] SC7: `cargo metadata --format-version=1 | jq '.packages[0].name'` returns `"stevedore"`

## Anti-Patterns (FORBIDDEN)

- NO renaming `cargo_test`, `cargo_clippy`, etc. tool names (they describe cargo subcommands)
- NO creating backwards-compatibility aliases for `cargo-mcp`
- NO changing the `.bones/` task ID prefix from `cm-` (historical, not branding)
