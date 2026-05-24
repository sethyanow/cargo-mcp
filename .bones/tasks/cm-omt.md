---
id: cm-omt
title: Add process timeout to execute_cargo_command
status: open
type: task
priority: 2
phase: design
---


## Context

Flagged during adversarial reflection on cm-1k8. `cmd.output()` at
`src/tools/cargo_utils.rs:39` blocks the calling thread indefinitely if cargo
hangs (e.g., waiting for interactive input, stuck on a network fetch, infinite
loop in a test binary). Because `execute_cargo_command` runs on the MCP server
thread, a hung cargo process stalls the entire server — no tool calls can be
handled until the process terminates or the server is killed.

All 12 tool files delegate to this single function, so the fix is entirely
contained in `cargo_utils.rs`. No callers need changes.

## Requirements

R1. `execute_cargo_command` MUST NOT block indefinitely. It MUST return an
    `Err` if the child process does not complete within the configured timeout.

R2. The timeout duration MUST be expressed as a named constant, not a magic
    number embedded in the call site.

R3. On timeout, the child process MUST be killed before the error is returned.
    Orphaned children accumulate across tool invocations and are unacceptable
    for a long-running MCP server.

R4. The timeout error message MUST include the command name and the timeout
    duration in seconds so the caller can surface it meaningfully.

R5. Normal (non-timeout) behavior MUST be preserved: stdout, stderr, exit
    code, and the formatted output string are identical to the current
    implementation.

## Implementation Notes

**Fix location:** `src/tools/cargo_utils.rs` — `execute_cargo_command` only.
`cmd.output()` on line 39 becomes a spawn + timed wait. No other files change.

**Approach — thread + channel (zero new deps):**

```rust
use std::sync::mpsc;
use std::time::Duration;
use std::process::Stdio;

const CARGO_COMMAND_TIMEOUT_SECS: u64 = 600; // 10 minutes

let mut child = cmd
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()?;

let (tx, rx) = mpsc::channel();
std::thread::spawn(move || {
    let _ = tx.send(child.wait_with_output());
});

let output = match rx.recv_timeout(Duration::from_secs(CARGO_COMMAND_TIMEOUT_SECS)) {
    Ok(result) => result?,
    Err(_elapsed) => {
        // NOTE: child moved into thread; kill via OS signal on unix.
        // Orphan on other platforms — acceptable for a dev tool.
        #[cfg(unix)]
        unsafe { libc::kill(pid as libc::pid_t, libc::SIGKILL); }
        anyhow::bail!(
            "cargo {command_name} timed out after {CARGO_COMMAND_TIMEOUT_SECS}s"
        );
    }
};
```

**Kill problem:** `child` moves into the thread; we can't call `child.kill()`
from the timeout branch without keeping a separate handle. Two clean options:

- **Option A (recommended):** Add `wait-timeout = "0.2"` — a tiny, stable,
  cross-platform crate that extends `Child` with `wait_timeout(&mut self,
  timeout) -> Result<Option<ExitStatus>>`, keeping ownership in the caller so
  `child.kill()` is reachable. Zero transitive deps.

- **Option B (zero new deps):** Thread + channel. Store `child.id()` (PID)
  before moving child into thread. On timeout, send SIGKILL via `libc::kill`
  (`#[cfg(unix)]`). Requires adding `libc` to deps or accepting non-kill on
  Windows. Less clean but no additional crate.

Decide at implementation time; either satisfies R3 on Unix targets. Document
the choice in a code comment citing R3.

**Test strategy:** `execute_cargo_command` is untested as an execution path.
The new test must actually spawn a subprocess and verify timeout fires.

```rust
#[cfg(unix)]
#[test]
fn execute_cargo_command_times_out() {
    use std::process::Command;
    use std::path::PathBuf;

    // Temporarily use a 1-second timeout constant, or patch the function
    // to accept an explicit timeout in tests only (cfg(test) parameter).
    // Simplest: extract timeout to a fn parameter with a default wrapper.

    let cmd = Command::new("sleep").arg("60");
    let result = execute_cargo_command_with_timeout(
        cmd,
        &PathBuf::from("/tmp"),
        "sleep",
        Duration::from_secs(1),
    );
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("timed out"), "expected timeout message, got: {msg}");
    assert!(msg.contains("sleep"), "expected command name in error");
}
```

For the test to work without coupling test-only state into production code,
split into a private `execute_cargo_command_with_timeout(cmd, path, name,
timeout)` and a public `execute_cargo_command` that calls it with
`CARGO_COMMAND_TIMEOUT_SECS`. Tests call the internal function directly.

## Success Criteria

- [ ] SC1: `cmd.output()` at `cargo_utils.rs:39` is replaced with a spawn +
      timed wait that returns `Err` after `CARGO_COMMAND_TIMEOUT_SECS` seconds.

- [ ] SC2: `CARGO_COMMAND_TIMEOUT_SECS` (or equivalent named constant) exists
      in `cargo_utils.rs` and is the sole place the duration is defined.

- [ ] SC3: On timeout, the child process is killed (not orphaned) on Unix
      targets. Mechanism must be documented in a code comment.

- [ ] SC4: Timeout error includes command name and duration, e.g.
      `"cargo test timed out after 600s"`.

- [ ] SC5: Regression test `execute_cargo_command_times_out` (unix-gated)
      passes: spawns `sleep 60` with a 1-second timeout and asserts `Err`
      containing `"timed out"` and the command name.

- [ ] SC6: All existing 71 tests still pass. `cargo clippy --all-targets` clean.
      `cargo fmt --check` clean.

## Log

- [2026-05-24T13:48:34Z] [claude-code] Scoped via LSP: fix is fully contained in execute_cargo_command (cargo_utils.rs:32). All 12 callers unchanged. Two implementation options documented (wait-timeout crate vs thread+channel+libc). Test strategy: extract execute_cargo_command_with_timeout for testability.
