---
id: cm-x8c
title: Managed target directory with disk usage reporting
status: open
type: task
priority: 2
depends_on: [cm-qnh, cm-dvs]
phase: design
---





## Context

The server runs cargo commands in user project directories but has no
awareness of the disk space those commands consume. Build artifacts
accumulate in `target/` dirs across projects with no visibility or
cleanup. Agents can fill drives without knowing it.

`cargo metadata --format-version=1` returns `target_directory` — the
resolved path cargo will use, respecting `CARGO_TARGET_DIR`,
`.cargo/config.toml`, etc. This gives us the actual target path with
zero guessing.

Every tool already accepts `cargo_env` for per-call env vars, and the
MCP config `env` field sets server-wide env vars. `CARGO_TARGET_DIR`
already works through these mechanisms — what's missing is the server
using them by default and providing visibility.

## Requirements

R1. The server MUST auto-set `CARGO_TARGET_DIR` to a per-project
    managed location in `create_cargo_command` unless one is already
    set (via `cargo_env`, process env, or `.cargo/config.toml`).
    Path: `<cache_dir>/stevedore/target/<hash>/` where `<cache_dir>`
    follows XDG conventions (`dirs::cache_dir()`) and `<hash>` is
    derived from the project's absolute path. Per-project isolation
    ensures `cargo clean` in one project doesn't nuke another's
    artifacts.

R2. A new `cargo_info` tool (or similar) MUST report disk usage stats
    for the current project. Use `cargo metadata --format-version=1` to
    resolve `target_directory`, then report its size on disk.

R3. `cargo_clean` MUST report the size freed after cleaning. Before/after
    delta from the same `target_directory` path.

## Scope

### Files that change

| File | What changes |
|------|-------------|
| `src/tools/cargo_utils.rs` | `create_cargo_command` injects `CARGO_TARGET_DIR` if not already set |
| `src/state.rs` | Store managed target dir path, resolved at startup |
| `src/tools/cargo_info.rs` | New tool: runs `cargo metadata`, reports target dir size |
| `src/tools/cargo_clean.rs` | Report size freed (before/after) |
| `src/tools.rs` | Register `cargo_info` in `tools!` macro |

### Files that DON'T change
- Other tool files — they call `create_cargo_command` which handles injection
- `src/main.rs` — no changes needed

### Design notes
- `cargo metadata --format-version=1` returns JSON with `target_directory`
  and `workspace_root`. Gives the real resolved path.
- Disk size: `fs::metadata` walk or shell `du -sb`. Walk is portable,
  `du` is simpler. Start with `du` on unix, revisit if cross-platform
  matters.
- Managed target dir: `dirs::cache_dir()/stevedore/target/<hash>/`
  resolves to `~/Library/Caches/stevedore/target/<hash>` on macOS,
  `~/.cache/stevedore/target/<hash>` on Linux. Hash from absolute
  project path — short hex (e.g., first 12 chars of SHA-256) for
  filesystem friendliness.
- User override: if `CARGO_TARGET_DIR` already in process env or passed
  via `cargo_env`, don't inject. Respect user intent.

## Success Criteria

- [ ] SC1: With no user config, cargo commands write artifacts to
      `<cache_dir>/stevedore/target/<hash>/`, not `<project>/target/`
- [ ] SC2: `cargo_info` returns JSON with `target_directory` (string),
      `project_path` (string), and `size_bytes` (integer) fields
- [ ] SC3: `cargo_clean` output includes bytes/MB freed
- [ ] SC4: User-set `CARGO_TARGET_DIR` (via env or `cargo_env`) is
      respected — server does not override it
- [ ] SC5: All tests pass, clippy clean, fmt clean

## Anti-Patterns (FORBIDDEN)

- NO overriding a user-set `CARGO_TARGET_DIR`
- NO background threads or timers for auto-cleanup (agent decides when)
- NO threshold-based auto-clean in v1 (keep it simple — report, don't act)
