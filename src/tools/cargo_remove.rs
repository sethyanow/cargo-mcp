use crate::state::CargoTools;
use crate::tools::cargo_utils::{create_cargo_command, execute_cargo_command};
use anyhow::{anyhow, Result};
use mcplease::{
    traits::{Tool, WithExamples},
    types::Example,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Remove dependencies from Cargo.toml using cargo remove
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema, clap::Args)]
#[serde(rename = "cargo_remove")]
pub struct CargoRemove {
    /// List of dependencies to remove
    pub dependencies: Vec<String>,

    /// Optional package name (for workspaces)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub package: Option<String>,

    /// Remove from development dependencies
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub dev: Option<bool>,

    /// Optional Rust toolchain to use (e.g., 'stable', 'nightly', '1.70.0')
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub toolchain: Option<String>,

    /// Optional environment variables to set for the cargo command
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(skip)]
    pub cargo_env: Option<HashMap<String, String>>,

    /// Additional cargo arguments passed before any `--` separator
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(last = true)]
    pub extra_args: Option<Vec<String>>,
}

impl WithExamples for CargoRemove {
    fn examples() -> Vec<Example<Self>> {
        vec![
            Example {
                description: "Remove a dependency",
                item: Self {
                    dependencies: vec!["unused-crate".into()],
                    package: None,
                    dev: None,
                    toolchain: None,
                    cargo_env: None,
                    extra_args: None,
                },
            },
            Example {
                description: "Remove multiple dependencies",
                item: Self {
                    dependencies: vec!["old-lib".into(), "deprecated-crate".into()],
                    package: None,
                    dev: None,
                    toolchain: None,
                    cargo_env: None,
                    extra_args: None,
                },
            },
            Example {
                description: "Remove a dev dependency",
                item: Self {
                    dependencies: vec!["old-test-util".into()],
                    package: None,
                    dev: Some(true),
                    toolchain: None,
                    cargo_env: None,
                    extra_args: None,
                },
            },
        ]
    }
}

impl CargoRemove {
    /// Build the cargo remove argument list.
    pub fn build_args(&self) -> Vec<String> {
        let mut args = vec!["remove".to_string()];

        if let Some(ref package) = self.package {
            args.push("--package".to_string());
            args.push(package.clone());
        }

        if self.dev.unwrap_or(false) {
            args.push("--dev".to_string());
        }

        // Add the dependencies to remove
        for dep in &self.dependencies {
            args.push(dep.clone());
        }

        if let Some(ref extra) = self.extra_args {
            args.extend(extra.iter().cloned());
        }

        args
    }
}

impl Tool<CargoTools> for CargoRemove {
    fn execute(self, state: &mut CargoTools) -> Result<String> {
        if self.dependencies.is_empty() {
            return Err(anyhow!("No dependencies specified"));
        }

        let project_path = state.ensure_rust_project(None)?;
        let args = self.build_args();

        // Use toolchain from args, session default, or none
        let toolchain = self
            .toolchain
            .or_else(|| state.get_default_toolchain(None).unwrap_or(None));

        let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let cmd = create_cargo_command(&args_refs, toolchain.as_deref(), self.cargo_env.as_ref());
        execute_cargo_command(cmd, &project_path, "cargo remove")
    }
}
