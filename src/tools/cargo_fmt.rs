use crate::state::CargoTools;
use crate::tools::cargo_utils::{create_cargo_command, execute_cargo_command};
use anyhow::Result;
use mcplease::{
    traits::{Tool, WithExamples},
    types::Example,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Run cargo fmt to check or fix code formatting
#[derive(Debug, Default, Serialize, Deserialize, schemars::JsonSchema, clap::Args)]
#[serde(rename = "cargo_fmt")]
pub struct CargoFmt {
    /// Whether to run in check mode (default true). When true, passes --check
    /// to cargo fmt. When false, runs cargo fmt in write mode (modifies files).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub check: Option<bool>,

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
    #[arg(skip)]
    pub extra_args: Option<Vec<String>>,
}

impl CargoFmt {
    /// Build the cargo fmt argument list.
    pub fn build_args(&self) -> Vec<String> {
        let mut args = vec!["fmt".to_string()];
        if self.check.unwrap_or(true) {
            args.push("--check".to_string());
        }
        if let Some(ref extra) = self.extra_args {
            args.extend(extra.iter().cloned());
        }
        args
    }
}

impl WithExamples for CargoFmt {
    fn examples() -> Vec<Example<Self>> {
        vec![
            Example {
                description: "Check formatting in current project",
                item: Self {
                    check: None,
                    toolchain: None,
                    cargo_env: None,
                    extra_args: None,
                },
            },
            Example {
                description: "Check formatting with nightly toolchain",
                item: Self {
                    check: None,
                    toolchain: Some("nightly".into()),
                    cargo_env: None,
                    extra_args: None,
                },
            },
            Example {
                description: "Fix formatting (write mode)",
                item: Self {
                    check: Some(false),
                    toolchain: None,
                    cargo_env: None,
                    extra_args: None,
                },
            },
        ]
    }
}

impl Tool<CargoTools> for CargoFmt {
    fn execute(self, state: &mut CargoTools) -> Result<String> {
        let project_path = state.ensure_rust_project(None)?;

        let args = self.build_args();
        let label = if self.check.unwrap_or(true) {
            "cargo fmt --check"
        } else {
            "cargo fmt"
        };

        // Use toolchain from args, session default, or none
        let toolchain = self.toolchain
            .or_else(|| state.get_default_toolchain(None).unwrap_or(None));

        let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let cmd = create_cargo_command(&args_refs, toolchain.as_deref(), self.cargo_env.as_ref());
        execute_cargo_command(cmd, &project_path, label)
    }
}
