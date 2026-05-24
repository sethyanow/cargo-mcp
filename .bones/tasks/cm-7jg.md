---
id: cm-7jg
title: Rename cargo-mcp to stevedore
status: active
type: task
priority: 1
owner: claude-code
phase: design
---


## Context

Renaming the project from `cargo-mcp` (upstream name) to `stevedore` for
publication as an independent crate. Upstream (jbr/cargo-mcp) is abandoned.
Name chosen: "stevedore" — the person who loads and unloads cargo.

## Requirements

R1. Cargo.toml package name MUST be `cargo-stevedore` (crates.io name,
    enables `cargo stevedore` subcommand convention). Binary name is
    `cargo-stevedore` (default from package name — no `[[bin]]` override).

R2. All documentation branding MUST say `stevedore`, not `cargo-mcp`.
    MCP client config examples MUST show `"command": "cargo-stevedore"`.
    Install command: `cargo install cargo-stevedore`.

R3. Cargo.toml `authors` MUST list only the current maintainer (Seth).
    `repository` and `documentation` MUST point to the fork. Description
    MUST be updated. Original author credited in README, not Cargo.toml.

R4. Session file path MUST move from `~/.ai-tools/sessions/cargo-mcp.json`
    to XDG-compliant `dirs::data_dir() / "stevedore" / "session.json"`.
    Old file is abandoned (not migrated). `dirs` crate already a dependency.

R5. The `server_info!()` macro in main.rs reads from Cargo.toml — verify
    it picks up the new name automatically (no code change expected).

## Scope

### Files that change

| File | What changes |
|------|-------------|
| `Cargo.toml` | `name`, `description`, `repository`, `documentation`, `authors`, `keywords` |
| `README.md` | All ~10 `cargo-mcp` references, install command, MCP config examples, repo URLs |
| `CLAUDE.md` | ~2 references to `cargo-mcp` in project description |
| `src/state.rs` | Session path: `~/.ai-tools/sessions/cargo-mcp.json` → `dirs::data_dir()/stevedore/session.json`; env var: fallback old name with deprecation warning |
| `src/tools/set_working_directory.rs` | Doc comment mentioning `cargo-mcp` (line 13) |
| `src/main.rs` | INSTRUCTIONS string — currently generic, no change needed |

### Files that DON'T change
- `src/tools/cargo_*.rs` — tool names stay `cargo_*` (they wrap cargo commands, not the server)
- `.bones/` — historical records, leave as-is
- `CHANGELOG.md` — upstream URLs are historical attribution, not branding
- Test files — no `cargo-mcp` references in test assertions

### Out of scope
- GitHub repo rename (manual, separate from code)
- crates.io publishing (separate step after rename merges)
- Changing tool names from `cargo_*` (those describe the cargo subcommands they wrap)

## Success Criteria

- [x] SC1: `cargo install --path .` produces a `cargo-stevedore` binary
- [x] SC2: `cargo-stevedore serve` starts the MCP server
- [x] SC3: `cargo stevedore serve` works (cargo subcommand delegation)
- [x] SC4: README branding says "stevedore", not `cargo-mcp`
      (references to upstream for attribution are fine)
- [x] SC5: MCP config examples use `"command": "cargo-stevedore"`
- [x] SC6: Session file writes to `dirs::data_dir()/stevedore/session.json`
- [x] SC7: All tests pass (79/79), clippy clean, fmt clean
- [x] SC8: `cargo metadata --format-version=1 --no-deps | jq '.packages[0].name'` returns `"cargo-stevedore"`
- [x] SC9: Cargo.toml `authors` is Seth only; `repository` and `documentation` point to fork
- [x] SC10: `STEVEDORE_DEFAULT_TOOLCHAIN` env var replaces `CARGO_MCP_DEFAULT_TOOLCHAIN`

## Key Considerations (SRE)

- `server_info!()` uses `env!("CARGO_PKG_NAME")` — verified, picks up new name automatically
- CHANGELOG.md has upstream jbr/cargo-mcp URLs — these are historical, leave untouched
- main.rs INSTRUCTIONS string is generic ("Cargo operations for Rust projects") — no change needed
- Session file path change (R4) is code logic, needs regression test
- Doc files (README, CLAUDE.md, Cargo.toml) are non-logic — TDD escape hatch applies

## Failure Catalog (Adversarial)

**State Corruption: Session file path**
- Assumption: Fresh install, no prior state
- Betrayal: Existing `~/.ai-tools/sessions/cargo-mcp.json` with saved toolchain defaults orphaned. New `stevedore.json` starts empty.
- Consequence: User's configured default toolchain silently stops working after upgrade
- Mitigation: DECIDED (c) — accept loss. Low user count, trivially re-set. Path also moves to XDG-compliant location.

**Temporal Betrayal: Env var rename**
- Assumption: Users will update env var in shell config
- Betrayal: `CARGO_MCP_DEFAULT_TOOLCHAIN` in `.bashrc` silently ignored. No error, no warning.
- Consequence: Default toolchain silently stops working
- Mitigation: DECIDED (a) — check `STEVEDORE_DEFAULT_TOOLCHAIN` first, fall back to `CARGO_MCP_DEFAULT_TOOLCHAIN` with `log::warn!` deprecation message

**Encoding Boundaries: README text replacement**
- Assumption: All `cargo-mcp` strings should become stevedore-branded
- Betrayal: Upstream attribution URLs (`github.com/jbr/cargo-mcp`) must stay as-is
- Consequence: Broken upstream links, lost provenance
- Mitigation: Replace per-occurrence, not globally. SC4 already allows attribution references.

**Temporal Betrayal: Binary coexistence**
- Assumption: Clean install
- Betrayal: Old `cargo-mcp` binary stays in `~/.cargo/bin/`. Both binaries exist.
- Consequence: User runs old binary expecting new behavior
- Mitigation: Document `cargo uninstall cargo-mcp` in README upgrade instructions. Not a code concern.

## Anti-Patterns (FORBIDDEN)

- NO renaming `cargo_test`, `cargo_clippy`, etc. tool names (they describe cargo subcommands)
- NO creating backwards-compatibility aliases for `cargo-mcp`
- NO changing the `.bones/` task ID prefix from `cm-` (historical, not branding)
