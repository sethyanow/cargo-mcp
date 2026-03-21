use crate::state::CargoTools;
use crate::tools::cargo_utils::{create_cargo_command, execute_cargo_command};
use anyhow::Result;
use mcplease::{
    traits::{Tool, WithExamples},
    types::Example,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Build the project with cargo build
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema, clap::Args)]
#[serde(rename = "cargo_build")]
pub struct CargoBuild {
    /// Optional package name to build (for workspaces)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub package: Option<String>,

    /// Build in release mode
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub release: Option<bool>,

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

impl WithExamples for CargoBuild {
    fn examples() -> Vec<Example<Self>> {
        vec![
            Example {
                description: "Build the project in debug mode",
                item: Self {
                    package: None,
                    release: None,
                    toolchain: None,
                    cargo_env: None,
                    extra_args: None,
                },
            },
            Example {
                description: "Build the project in release mode",
                item: Self {
                    package: None,
                    release: Some(true),
                    toolchain: None,
                    cargo_env: None,
                    extra_args: None,
                },
            },
            Example {
                description: "Build a specific package",
                item: Self {
                    package: Some("my-lib".into()),
                    release: None,
                    toolchain: None,
                    cargo_env: None,
                    extra_args: None,
                },
            },
            Example {
                description: "Build with nightly toolchain",
                item: Self {
                    package: None,
                    release: None,
                    toolchain: Some("nightly".into()),
                    cargo_env: None,
                    extra_args: None,
                },
            },
        ]
    }
}

impl CargoBuild {
    /// Build the cargo build argument list.
    pub fn build_args(&self) -> Vec<String> {
        let mut args = vec!["build".to_string()];

        if let Some(ref package) = self.package {
            args.push("--package".to_string());
            args.push(package.clone());
        }

        if self.release.unwrap_or(false) {
            args.push("--release".to_string());
        }

        if let Some(ref extra) = self.extra_args {
            args.extend(extra.iter().cloned());
        }

        args
    }
}

impl Tool<CargoTools> for CargoBuild {
    fn execute(self, state: &mut CargoTools) -> Result<String> {
        let project_path = state.ensure_rust_project(None)?;
        let args = self.build_args();

        // Use toolchain from args, session default, or none
        let toolchain = self
            .toolchain
            .or_else(|| state.get_default_toolchain(None).unwrap_or(None));

        let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let cmd = create_cargo_command(&args_refs, toolchain.as_deref(), self.cargo_env.as_ref());
        execute_cargo_command(cmd, &project_path, "cargo build")
    }
}
