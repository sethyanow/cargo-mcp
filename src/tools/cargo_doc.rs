use crate::state::CargoTools;
use crate::tools::cargo_utils::{create_cargo_command, execute_cargo_command};
use anyhow::Result;
use mcplease::{
    traits::{Tool, WithExamples},
    types::Example,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Run cargo doc to generate documentation
#[derive(Debug, Default, Serialize, Deserialize, schemars::JsonSchema, clap::Args)]
#[serde(rename = "cargo_doc")]
pub struct CargoDoc {
    /// Optional package name to document (for workspaces)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub package: Option<String>,

    /// Skip building documentation for dependencies (default true).
    /// When true, passes --no-deps to cargo doc.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub no_deps: Option<bool>,

    /// Document private items (default false).
    /// When true, passes --document-private-items to cargo doc.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub document_private_items: Option<bool>,

    /// Optional Rust toolchain to use (e.g., 'stable', 'nightly', '1.70.0')
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub toolchain: Option<String>,

    /// Optional environment variables to set for the cargo command
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(skip)]
    pub cargo_env: Option<HashMap<String, String>>,
}

impl CargoDoc {
    /// Build the cargo doc argument list.
    pub fn build_args(&self) -> Vec<String> {
        let mut args = vec!["doc".to_string()];
        if let Some(ref package) = self.package {
            args.push("--package".to_string());
            args.push(package.clone());
        }
        if self.no_deps.unwrap_or(true) {
            args.push("--no-deps".to_string());
        }
        if self.document_private_items.unwrap_or(false) {
            args.push("--document-private-items".to_string());
        }
        args
    }
}

impl WithExamples for CargoDoc {
    fn examples() -> Vec<Example<Self>> {
        vec![
            Example {
                description: "Generate docs for current project (skipping dependencies)",
                item: Self {
                    package: None,
                    no_deps: None,
                    document_private_items: None,
                    toolchain: None,
                    cargo_env: None,
                },
            },
            Example {
                description: "Generate docs for a specific package",
                item: Self {
                    package: Some("my-lib".into()),
                    no_deps: None,
                    document_private_items: None,
                    toolchain: None,
                    cargo_env: None,
                },
            },
            Example {
                description: "Generate docs including private items",
                item: Self {
                    package: None,
                    no_deps: None,
                    document_private_items: Some(true),
                    toolchain: None,
                    cargo_env: None,
                },
            },
            Example {
                description: "Generate docs including dependencies",
                item: Self {
                    package: None,
                    no_deps: Some(false),
                    document_private_items: None,
                    toolchain: None,
                    cargo_env: None,
                },
            },
        ]
    }
}

impl Tool<CargoTools> for CargoDoc {
    fn execute(self, state: &mut CargoTools) -> Result<String> {
        let project_path = state.ensure_rust_project(None)?;

        let args = self.build_args();

        // Use toolchain from args, session default, or none
        let toolchain = self
            .toolchain
            .or_else(|| state.get_default_toolchain(None).unwrap_or(None));

        let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let cmd = create_cargo_command(&args_refs, toolchain.as_deref(), self.cargo_env.as_ref());
        execute_cargo_command(cmd, &project_path, "cargo doc")
    }
}
