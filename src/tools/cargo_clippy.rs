use crate::state::CargoTools;
use crate::tools::cargo_utils::{create_cargo_command, execute_cargo_command};
use anyhow::Result;
use mcplease::{
    traits::{Tool, WithExamples},
    types::Example,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Run cargo clippy for linting suggestions
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema, clap::Args)]
#[serde(rename = "cargo_clippy")]
pub struct CargoClippy {
    /// Optional package name to lint (for workspaces)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub package: Option<String>,

    /// Optional Rust toolchain to use (e.g., 'stable', 'nightly', '1.70.0')
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub toolchain: Option<String>,

    /// Apply suggested fixes automatically
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub fix: Option<bool>,

    /// Run clippy on all targets (tests, examples, benchmarks)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub all_targets: Option<bool>,

    /// Optional environment variables to set for the cargo command
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(skip)]
    pub cargo_env: Option<HashMap<String, String>>,
}

impl WithExamples for CargoClippy {
    fn examples() -> Vec<Example<Self>> {
        vec![
            Example {
                description: "Basic cargo clippy in current project",
                item: Self {
                    package: None,
                    toolchain: None,
                    fix: None,
                    cargo_env: None,
                    all_targets: None,
                },
            },
            Example {
                description: "Run clippy on a specific package",
                item: Self {
                    package: Some("my-lib".into()),
                    toolchain: None,
                    fix: None,
                    cargo_env: None,
                    all_targets: None,
                },
            },
            Example {
                description: "Run clippy with automatic fixes",
                item: Self {
                    package: None,
                    toolchain: None,
                    fix: Some(true),
                    cargo_env: None,
                    all_targets: None,
                },
            },
            Example {
                description: "Run clippy with nightly toolchain",
                item: Self {
                    package: None,
                    toolchain: Some("nightly".into()),
                    fix: None,
                    cargo_env: None,
                    all_targets: None,
                },
            },
            Example {
                description: "Run clippy on all targets (tests, examples, benchmarks)",
                item: Self {
                    package: None,
                    toolchain: None,
                    fix: None,
                    cargo_env: None,
                    all_targets: Some(true),
                },
            },
        ]
    }
}

impl CargoClippy {
    /// Build the cargo clippy argument list.
    pub fn build_args(&self) -> Vec<String> {
        let mut args = vec!["clippy".to_string()];

        if let Some(ref package) = self.package {
            args.push("--package".to_string());
            args.push(package.clone());
        }

        if self.fix.unwrap_or(false) {
            args.push("--fix".to_string());
        }

        if self.all_targets.unwrap_or(false) {
            args.push("--all-targets".to_string());
        }

        // Add clippy arguments
        args.extend_from_slice(&["--".to_string(), "-D".to_string(), "warnings".to_string()]);

        args
    }
}

impl Tool<CargoTools> for CargoClippy {
    fn execute(self, state: &mut CargoTools) -> Result<String> {
        let project_path = state.ensure_rust_project(None)?;
        let args = self.build_args();

        // Use toolchain from args, session default, or none
        let toolchain = self.toolchain
            .or_else(|| state.get_default_toolchain(None).unwrap_or(None));

        let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let cmd = create_cargo_command(&args_refs, toolchain.as_deref(), self.cargo_env.as_ref());
        execute_cargo_command(cmd, &project_path, "cargo clippy")
    }
}
