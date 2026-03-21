# cargo-mcp

An MCP server that exposes Cargo commands as tools, so AI assistants can build, test, lint, and manage dependencies in Rust projects without arbitrary shell access.

Built on [mcplease](https://crates.io/crates/mcplease).

## Installation

```bash
cargo install cargo-mcp
```

## Configuration

Add cargo-mcp to your MCP client configuration. For Claude Desktop:

```json
{
  "mcpServers": {
    "cargo-mcp": {
      "command": "cargo-mcp",
      "args": ["serve"]
    }
  }
}
```

To pin a default Rust toolchain, set the `CARGO_MCP_DEFAULT_TOOLCHAIN` environment variable:

```json
{
  "mcpServers": {
    "cargo-mcp": {
      "command": "cargo-mcp",
      "args": ["serve"],
      "env": { "CARGO_MCP_DEFAULT_TOOLCHAIN": "stable" }
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

Every tool accepts `toolchain` (e.g., `"nightly"`) and `cargo_env` (a map of environment variables) parameters.

## How it works

Each tool maps to a cargo subcommand. The server validates that the working directory contains a `Cargo.toml`, builds the appropriate `cargo` invocation, and returns stdout/stderr. No arbitrary command execution is possible -- only the tools listed above are available, and all run in the specified project directory.

When a `toolchain` is provided, commands run through `rustup run <toolchain> cargo ...` rather than bare `cargo`.

## License

MIT OR Apache-2.0
