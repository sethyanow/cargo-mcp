---
id: cm-viy
title: Add CargoDoc tool (R4)
status: open
type: task
parent: cm-paw
---



## Context
Phase 1 sub-epic cm-paw, parent epic cm-1k8. Fourth and final implementation task.
New tool — `src/tools/cargo_doc.rs` doesn't exist. Follows established patterns from CargoCheck (simplest tool) and CargoClippy/CargoTest/CargoFmt (build_args pattern).

**Blocked by:** cm-f9o (closed) — sequential per epic flow
**Unlocks:** Phase 1 completion (all R1-R4 implementation tasks done, only acceptance task remains)

## Requirements
R4: New CargoDoc tool with `package`, `toolchain`, `cargo_env` (standard params), `no_deps` (default true, passes `--no-deps`), and `document_private_items` (default false, passes `--document-private-items`). Runs `cargo doc`.

## Implementation
1. Write failing test `doc_default_produces_no_deps` — construct `CargoDoc::default()`, call `build_args()`, assert args are `["doc", "--no-deps"]`. Expected RED: compile error (no `CargoDoc` struct).
2. Create `src/tools/cargo_doc.rs` with `CargoDoc` struct: derives `Debug, Default, Serialize, Deserialize, schemars::JsonSchema, clap::Args`. Fields: `package: Option<String>`, `no_deps: Option<bool>`, `document_private_items: Option<bool>`, `toolchain: Option<String>`, `cargo_env: Option<HashMap<String, String>>`. All with standard `skip_serializing_if` and `arg` attributes. `#[serde(rename = "cargo_doc")]`. Add `build_args()` stub returning `["doc", "--no-deps"]`. Add tools! macro entry: `(CargoDoc, cargo_doc, "cargo_doc")`. Stub `WithExamples` (empty vec) and `Tool<CargoTools>` impl (minimal). Verify step 1 test passes GREEN.
3. Write test `doc_with_deps_no_flag` — `no_deps: Some(false)`, assert args are `["doc"]` (no `--no-deps`). Expected RED: stub always returns `--no-deps`.
4. GREEN: implement `no_deps` branching in `build_args()`. `self.no_deps.unwrap_or(true)` → push `--no-deps`.
5. Write test `doc_private_items_flag` — `document_private_items: Some(true)`, assert args contain `--document-private-items`. Expected RED: not implemented yet.
6. GREEN: implement `document_private_items` branching. `self.document_private_items.unwrap_or(false)` → push `--document-private-items`.
7. Write test `doc_with_package` — `package: Some("foo")`, assert args contain `["--package", "foo"]`. Expected RED: package not handled in build_args yet.
8. GREEN: implement package handling in `build_args()`.
9. Write test `doc_all_fields_set` — all params populated, assert correct arg ordering, verify toolchain NOT in build_args. Should pass if prior steps are correct.
10. Refactor `execute()` to call `build_args()`, convert to `Vec<&str>`, pass to `create_cargo_command`. Follow CargoFmt pattern (call build_args before consuming self.toolchain).
11. Fill `WithExamples` with examples: default (no-deps doc), with package, with private items, with deps (`no_deps: Some(false)`).
12. Full suite + clippy + fmt, commit, push.

## Success Criteria
- [ ] `CargoDoc` struct exists in `src/tools/cargo_doc.rs`
- [ ] MCP tool name is `cargo_doc`
- [ ] Registered in `tools!` macro
- [ ] Default behavior (no_deps: None) passes `--no-deps`
- [ ] `no_deps: Some(false)` omits `--no-deps`
- [ ] `document_private_items: Some(true)` passes `--document-private-items`
- [ ] `package: Some(x)` passes `--package x`
- [ ] `build_args()` method exists and is tested
- [ ] `cargo test` passes (all 21+ tests)
- [ ] Examples include all param combinations

## Anti-Patterns
- NO defaulting `no_deps` to false (docs without `--no-deps` rebuild all dependencies — slow and usually unwanted)
- NO defaulting `document_private_items` to true (private items are rarely needed for MCP consumers)
- NO tautological tests that only verify struct field values
- NO end-to-end tests that invoke cargo doc
- NO skipping `build_args()` extraction (established project pattern)

## Key Considerations
- `cargo doc` doesn't use `--` separator — all flags are cargo-level. Simpler than clippy/test.
- `--no-deps` is the safe default because building deps docs is slow and rarely useful in MCP context
- `--document-private-items` uses a double-dash long flag (not `--private`)
- Field ordering in struct: domain-specific params first (no_deps, document_private_items), then standard (package, toolchain, cargo_env) — matches how R4 spec lists them
