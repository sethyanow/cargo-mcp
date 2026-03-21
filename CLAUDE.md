# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

**Use the cargo-mcp MCP tools for all cargo operations** (build, check, clippy, test, fmt). Set the working directory first via `set_working_directory`, then call the appropriate tool. Do not shell out with `cargo` directly when an MCP tool exists.

```
cargo_check                                    # verify compilation
cargo_clippy                                   # lint (lib/bin by default)
cargo_clippy { all_targets: true }             # lint tests, examples, benchmarks too
cargo_test                                     # run all tests
cargo_test { test_name: "test_name" }          # run a single test
cargo_test { use_nextest: true }               # run via cargo-nextest
cargo_fmt                                      # check formatting (default: check mode)
cargo_fmt { check: false }                     # fix formatting (write mode)
cargo_doc                                      # generate docs (skips deps by default)
cargo_doc { document_private_items: true }     # include private items
cargo_doc { no_deps: false }                   # include dependency docs
cargo_build                                    # build
```

If you must run cargo directly (e.g., for operations without an MCP tool), use `cargo +stable` or omit toolchain to use the default.

## LSP Usage

This is a Rust project — **use LSP (rust-analyzer) for all code navigation**:

- `workspaceSymbol` — find types/functions by name across the workspace (returns all crate structs, enums, traits, functions)
- `goToDefinition` — jump to where a symbol is defined (works into dependencies, e.g., resolving `Tool` to mcplease source)
- `findReferences` — find all usages of a symbol (e.g., find every file importing `create_cargo_command`)
- `hover` — get type info and docs without reading the file (e.g., see `CargoTools` struct fields and doc comments)
- `documentSymbol` — list all symbols in a file with types and signatures (e.g., all methods on `CargoTools`)
- `goToImplementation` — find trait implementations (e.g., find all `Tool<CargoTools>` impls across every tool file)
- `prepareCallHierarchy` — get the call hierarchy item at a position (needed before `incomingCalls`/`outgoingCalls`)
- `incomingCalls` — find all functions that call a given function (e.g., every `execute()` that calls `create_cargo_command`)
- `outgoingCalls` — find all functions called by a given function (e.g., what stdlib calls `create_cargo_command` makes)

rust-analyzer needs ~30-60 seconds to index after session start. If LSP returns empty results early in a session, wait and retry.

## Architecture

This is an MCP (Model Context Protocol) server that exposes cargo commands as tools. Built on the [mcplease](https://crates.io/crates/mcplease) framework.

### Key framework concepts

- **`mcplease::traits::Tool<State>`** — trait every tool implements. Has one method: `fn execute(self, state: &mut State) -> Result<String>`. The `State` type here is `CargoTools`.
- **`mcplease::traits::WithExamples`** — trait providing usage examples shown to MCP clients. Returns `Vec<Example<Self>>`.
- **`mcplease::tools!` macro** — registers tools in `src/tools.rs`. Format: `(StructName, module_name, "mcp_tool_name")`. Adding a tool requires an entry here.
- **`mcplease::run`** — entry point in `main.rs`. Starts the MCP server with the registered tools.

### Project structure

- `src/main.rs` — entry point, server startup, instructions string
- `src/state.rs` — `CargoTools` struct (shared state). Manages working directory (per-process, in-memory) and session data (toolchain defaults, persisted to `~/.ai-tools/sessions/cargo-mcp.json`)
- `src/tools.rs` — tool registration via `tools!` macro
- `src/tools/cargo_utils.rs` — `create_cargo_command` and `execute_cargo_command` helpers used by every tool
- `src/tools/cargo_*.rs` — one file per tool
- `src/tests.rs` — integration tests

### Tool pattern

Every tool follows the same structure:

1. **Struct** with `#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema, clap::Args)]` and `#[serde(rename = "tool_name")]`
2. **`WithExamples` impl** — usage examples
3. **`Tool<CargoTools>` impl** — `execute()` method that:
   - Calls `state.ensure_rust_project(None)?` to get the working directory
   - Resolves toolchain via `self.toolchain.or_else(|| state.get_default_toolchain(...))`
   - Builds an args vec (e.g., `["clippy", "--package", pkg, "--", "-D", "warnings"]`)
   - Passes to `create_cargo_command(&args, toolchain, env)` then `execute_cargo_command(cmd, &path, label)`

All tool structs include `extra_args: Option<Vec<String>>` for passing arbitrary cargo-level arguments (before any `--` separator). This is spliced into the args vec before tool-specific trailing args.

All optional fields use `Option<T>` with `#[serde(skip_serializing_if = "Option::is_none")]`.

### State isolation

Working directory is **per-process** (in-memory `SessionStore`), not persisted. This prevents state bleed between concurrent MCP server instances (e.g., different worktrees). Toolchain defaults are persisted to disk.

## Active Work

See `.bones/` for tracked epics and tasks. Use `bn list` / `bn show <id>` to review current work.
