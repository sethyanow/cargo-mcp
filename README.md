# stevedore

An MCP server exposing cargo commands as tools for AI agents. Build, test, lint, and manage dependencies in Rust projects without arbitrary shell access.

Built on [mcplease](https://crates.io/crates/mcplease). Forked from [jbr/cargo-mcp](https://github.com/jbr/cargo-mcp).

## Enhancements

### New tool
- **`cargo_doc`** — generate documentation with `no_deps` (default true) and `document_private_items` options

### Tool enhancements
- **`cargo_clippy`** — `all_targets` param to lint tests, examples, and benchmarks
- **`cargo_test`** — `use_nextest` param to run tests via [cargo-nextest](https://nexte.st/) with correct arg mapping (`--no-capture` instead of `-- --nocapture`)
- **`cargo_fmt`** — renamed from `cargo_fmt_check`; `check` param (default true) controls check vs. write mode
- **`extra_args` on every tool** — pass arbitrary cargo flags (e.g., `--no-default-features`, `--features`, `--lib`) before any `--` separator

### Reliability
- **Process timeout** — cargo commands are bounded to 600 seconds; hung processes are killed and reaped rather than blocking the server indefinitely
- **Per-process working directory** — concurrent MCP server instances (e.g., different worktrees) no longer share directory state

### Code quality
- **Testable arg building** — all tools refactored to extract `build_args()` methods, separating command construction from execution
- **Full test suite** — covers arg building, nextest mapping, separator ordering, extra_args placement, timeout behavior, and edge cases

## Installation

```bash
cargo install cargo-stevedore
```

## Configuration

Add stevedore to your MCP client configuration. For Claude Desktop:

```json
{
  "mcpServers": {
    "stevedore": {
      "command": "cargo-stevedore",
      "args": ["serve"]
    }
  }
}
```

To pin a default Rust toolchain, set the `STEVEDORE_DEFAULT_TOOLCHAIN` environment variable:

```json
{
  "mcpServers": {
    "stevedore": {
      "command": "cargo-stevedore",
      "args": ["serve"],
      "env": { "STEVEDORE_DEFAULT_TOOLCHAIN": "stable" }
    }
  }
}
```

Individual tool calls can override this with the `toolchain` parameter.

## Tools

Call `set_working_directory` first to point at a Rust project (must contain `Cargo.toml`), then use any of the tools below.

| Tool | Purpose |
|------|---------|
| `cargo_check` | Verify code compiles |
| `cargo_clippy` | Run Clippy linter (`all_targets` to include tests/examples/benches) |
| `cargo_test` | Run tests (`test_name` to filter, `use_nextest` for cargo-nextest) |
| `cargo_fmt` | Check or fix formatting (`check` param, default true) |
| `cargo_doc` | Generate documentation (`no_deps` default true, `document_private_items`) |
| `cargo_build` | Build (debug or release mode) |
| `cargo_bench` | Run benchmarks |
| `cargo_run` | Run a binary or example |
| `cargo_add` | Add dependencies to Cargo.toml |
| `cargo_remove` | Remove dependencies from Cargo.toml |
| `cargo_update` | Update dependencies |
| `cargo_clean` | Remove build artifacts |

Every tool accepts `toolchain` (e.g., `"nightly"`), `cargo_env` (a map of environment variables), and `extra_args` (a list of additional cargo-level arguments) parameters.

`extra_args` passes arbitrary flags to cargo before any `--` separator. This covers flags without dedicated parameters, such as `--no-default-features`, `--features`, `--lib`, and `--all-targets`:

```json
{ "extra_args": ["--no-default-features", "--features", "serde"] }
```

## How it works

Each tool maps to a cargo subcommand. The server validates that the working directory contains a `Cargo.toml`, builds the appropriate `cargo` invocation, and returns stdout/stderr. No arbitrary command execution is possible -- only the tools listed above are available, and all run in the specified project directory.

When a `toolchain` is provided, commands run through `rustup run <toolchain> cargo ...` rather than bare `cargo`.

## License

MIT OR Apache-2.0
