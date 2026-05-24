---
id: cm-omt
title: Add process timeout to execute_cargo_command
status: closed
type: task
priority: 2
owner: claude-code
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

`Child::try_wait(&mut self)` is the stdlib's non-blocking exit poll. Takes
`&mut self` (not `self`), so we retain child ownership and can call
`child.kill()` on timeout. Cross-platform — no libc, no unsafe.

Two pipe-drain threads prevent pipe deadlock (same pattern `cmd.output()`
uses internally — just made explicit).

**Why `try_wait` over alternatives:**
- `wait_with_output(self)` consumes child → can't call `kill()` on timeout
- `wait-timeout` crate — unnecessary dep for what stdlib provides
- Thread + channel + libc — requires unsafe, unix-only
- `try_wait` retains ownership, `kill()` is cross-platform, zero deps

**Test strategy:** Extract `execute_cargo_command_with_timeout(cmd, path,
name, timeout: Duration)` as a non-pub fn. Public `execute_cargo_command`
calls it with `Duration::from_secs(CARGO_COMMAND_TIMEOUT_SECS)`. Test lives
in an inline `#[cfg(test)] mod tests` block inside `cargo_utils.rs` (only way
to access non-pub fns in this private module — `src/tests.rs` cannot reach them).

**Edge cases (documented, acceptable):**
- On timeout, partial stdout/stderr is lost (not captured in error). Per R4,
  error only needs command name + duration.
- Pipe drain threads continue briefly after bail (until EOF from dead child).
  Not a resource leak — `kill()` + `wait()` ensures child death, pipes close.

## Key Considerations (Failure Catalog)

**Temporal Betrayal: try_wait/kill race window**
- Assumption: Process is alive when we call `kill()` after `try_wait()` returns `None`
- Betrayal: Process exits between `try_wait()==None` and `kill()`. On Windows,
  `TerminateProcess` on an exited process returns error.
- Consequence: Spurious io::Error propagated instead of clean timeout message.
- Mitigation: Use `let _ = child.kill();` (best-effort) + `child.wait()?`
  (always succeeds — process is dead either way). R3 satisfied: process dead,
  zombie reaped. The kill is good-faith; `wait()` is the structural guarantee.

**Input Hostility: Zero-duration timeout**
- Assumption: Timeout is positive (hundreds of seconds in production)
- Betrayal: `Duration::ZERO` — deadline immediately expired, process killed on first poll.
- Consequence: "timed out after 0s" error, no output captured.
- Mitigation: Not runtime-guarded. Production constant is 600s, test uses 1s.
  Document in fn signature. No criterion needed.

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
- [2026-05-24T16:42:56Z] [claude-code] SRE refinement: (1) Fixed test location — must be inline #[cfg(test)] mod inside cargo_utils.rs (private module). (2) Fixed test code — mut cmd + owned pass. (3) Clarified timeout param as Duration. (4) Documented edge cases: partial output lost on timeout, drain threads brief lifecycle. All skeleton claims verified via LSP find_references (12 callers confirmed).
- [2026-05-24T16:46:57Z] [claude-code] Adversarial planning complete. Key finding: try_wait/kill race on Windows — child.kill() can fail if process exits between try_wait()==None and kill(). Fix: let _ = child.kill() (best-effort) + child.wait()? (structural reap). Also documented zero-duration edge case (acceptable, no guard needed).
- [2026-05-24T17:01:10Z] [claude-code] Implementation complete. 76 tests pass (71 original + 5 new). Adversarial stress test: normal completion, failed exit, spawn failure, immediate exit — all GREEN with Three-Question traces. No bugs found. Key gap caught: existing tests never exercised execute_cargo_command (only arg building). Now covered.
