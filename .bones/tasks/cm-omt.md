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
`cmd.output()` on line 39 becomes spawn + `try_wait` poll loop. No other
files change. Zero new dependencies.

**Approach — `try_wait()` poll with deadline (zero deps, no unsafe):**

`Child::try_wait(&mut self)` is the stdlib's non-blocking exit poll. Because
it takes `&mut self` (not `self`), we retain child ownership and can call
`child.kill()` natively on timeout. Cross-platform — no libc, no unsafe.

Two pipe-drain threads are required to prevent pipe deadlock (same threads
`cmd.output()` spawns internally — just made explicit).

```rust
use std::io::Read;
use std::process::Stdio;
use std::time::{Duration, Instant};
use std::thread;

const CARGO_COMMAND_TIMEOUT_SECS: u64 = 600; // 10 minutes

let mut child = cmd
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()?;

// Drain pipes in background to prevent pipe deadlock
let stdout_pipe = child.stdout.take().unwrap();
let stderr_pipe = child.stderr.take().unwrap();
let stdout_thread = thread::spawn(move || {
    let mut buf = Vec::new();
    let mut r = stdout_pipe;
    r.read_to_end(&mut buf).map(|_| buf)
});
let stderr_thread = thread::spawn(move || {
    let mut buf = Vec::new();
    let mut r = stderr_pipe;
    r.read_to_end(&mut buf).map(|_| buf)
});

// Poll for exit with deadline
let deadline = Instant::now() + Duration::from_secs(timeout_secs);
let status = loop {
    match child.try_wait()? {
        Some(status) => break status,
        None if Instant::now() >= deadline => {
            child.kill()?;
            child.wait()?; // reap zombie
            anyhow::bail!(
                "cargo {command_name} timed out after {timeout_secs}s"
            );
        }
        None => thread::sleep(Duration::from_millis(100)),
    }
};

let stdout_bytes = stdout_thread.join().unwrap()?;
let stderr_bytes = stderr_thread.join().unwrap()?;
```

**Why `try_wait` over the alternatives:**
- `wait_with_output(self)` consumes child → can't call `kill()` on timeout
- `wait-timeout` crate works but is an unnecessary dep for what stdlib provides
- Thread + channel + libc requires unsafe for kill and is unix-only
- `try_wait` retains ownership, `kill()` is cross-platform, zero deps

**Test strategy:** Extract `execute_cargo_command_with_timeout(cmd, path,
name, timeout)` as private fn. Public `execute_cargo_command` calls it with
`CARGO_COMMAND_TIMEOUT_SECS`. Tests call the internal fn with short timeouts.

```rust
#[cfg(unix)]
#[test]
fn execute_cargo_command_times_out() {
    use std::process::Command;
    use std::path::PathBuf;
    use std::time::Duration;

    let cmd = Command::new("sleep");
    // sleep doesn't exist in /tmp as a project but execute_cargo_command
    // sets current_dir — just needs a valid path
    let result = execute_cargo_command_with_timeout(
        cmd.arg("60"),
        &PathBuf::from("/tmp"),
        "sleep-test",
        Duration::from_secs(1),
    );
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("timed out"), "expected timeout, got: {msg}");
    assert!(msg.contains("sleep-test"), "expected command name, got: {msg}");
}
```

## Success Criteria

- [ ] SC1: `cmd.output()` at `cargo_utils.rs:39` is replaced with a spawn +
      timed wait that returns `Err` after `CARGO_COMMAND_TIMEOUT_SECS` seconds.

- [ ] SC2: `CARGO_COMMAND_TIMEOUT_SECS` (or equivalent named constant) exists
      in `cargo_utils.rs` and is the sole place the duration is defined.

- [ ] SC3: On timeout, the child process is killed (not orphaned) via
      `Child::kill()` (cross-platform). Zombie reaped via `child.wait()`.

- [ ] SC4: Timeout error includes command name and duration, e.g.
      `"cargo test timed out after 600s"`.

- [ ] SC5: Regression test `execute_cargo_command_times_out` (unix-gated)
      passes: spawns `sleep 60` with a 1-second timeout and asserts `Err`
      containing `"timed out"` and the command name.

- [ ] SC6: All existing 71 tests still pass. `cargo clippy --all-targets` clean.
      `cargo fmt --check` clean.

## Log

- [2026-05-24T13:48:34Z] [claude-code] Scoped via LSP: fix is fully contained in execute_cargo_command (cargo_utils.rs:32). All 12 callers unchanged. Two implementation options documented (wait-timeout crate vs thread+channel+libc). Test strategy: extract execute_cargo_command_with_timeout for testability.
- [2026-05-24T16:32:57Z] [claude-code] Replaced Options A/B with try_wait() poll approach. Zero new deps, no unsafe, cross-platform kill via Child::kill(). Pipe-drain threads mirror what stdlib output() does internally. User decision.
