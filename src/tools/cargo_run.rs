use crate::state::CargoTools;
use crate::tools::cargo_utils::{create_cargo_command, execute_cargo_command};
use anyhow::Result;
use mcplease::traits::{Tool, WithExamples};
use mcplease::types::Example;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Run a binary or example from the current package
#[derive(Default, Debug, Serialize, Deserialize, schemars::JsonSchema, clap::Args)]
#[serde(rename = "cargo_run")]
pub struct CargoRun {
    /// Optional package name to run from (for workspaces)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub package: Option<String>,

    /// Optional binary name to run (if package has multiple binaries)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub bin: Option<String>,

    /// Optional example name to run instead of a binary
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub example: Option<String>,

    /// Run in release mode (optimized)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub release: Option<bool>,

    /// Space-separated list of features to activate
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub features: Option<String>,

    /// Activate all available features
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub all_features: Option<bool>,

    /// Do not activate the `default` feature
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub no_default_features: Option<bool>,

    /// Arguments to pass to the binary after `--`
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub args: Option<Vec<String>>,

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

impl WithExamples for CargoRun {
    fn examples() -> Vec<Example<Self>> {
        vec![
            Example {
                description: "Run the default binary",
                item: Self::default(),
            },
            Example {
                description: "Run a specific binary",
                item: Self {
                    bin: Some("my-binary".into()),
                    ..Self::default()
                },
            },
            Example {
                description: "Run an example",
                item: Self {
                    example: Some("hello".into()),
                    ..Self::default()
                },
            },
            Example {
                description: "Run with arguments passed to the binary",
                item: Self {
                    args: Some(vec!["--verbose".into(), "input.txt".into()]),
                    ..Self::default()
                },
            },
            Example {
                description: "Run in release mode with specific features",
                item: Self {
                    release: Some(true),
                    features: Some("feature1 feature2".into()),
                    ..Self::default()
                },
            },
            Example {
                description: "Run a binary from a specific workspace package",
                item: Self {
                    package: Some("my-workspace-crate".into()),
                    bin: Some("worker".into()),
                    args: Some(vec!["--config".into(), "prod.toml".into()]),
                    ..Self::default()
                },
            },
        ]
    }
}

impl CargoRun {
    /// Build the cargo run argument list.
    pub fn build_args(&self) -> Vec<String> {
        let mut args = vec!["run".to_string()];

        if let Some(ref package) = self.package {
            args.push("--package".to_string());
            args.push(package.clone());
        }

        if let Some(ref bin) = self.bin {
            args.push("--bin".to_string());
            args.push(bin.clone());
        }

        if let Some(ref example) = self.example {
            args.push("--example".to_string());
            args.push(example.clone());
        }

        if self.release.unwrap_or(false) {
            args.push("--release".to_string());
        }

        if let Some(ref features) = self.features {
            args.push("--features".to_string());
            args.push(features.clone());
        }

        if self.all_features.unwrap_or(false) {
            args.push("--all-features".to_string());
        }

        if self.no_default_features.unwrap_or(false) {
            args.push("--no-default-features".to_string());
        }

        if let Some(ref extra) = self.extra_args {
            args.extend(extra.iter().cloned());
        }

        // Add separator and binary arguments if provided
        if let Some(ref binary_args) = self.args
            && !binary_args.is_empty()
        {
            args.push("--".to_string());
            for arg in binary_args {
                args.push(arg.clone());
            }
        }

        args
    }
}

impl Tool<CargoTools> for CargoRun {
    fn execute(self, state: &mut CargoTools) -> Result<String> {
        let project_path = state.ensure_rust_project(None)?;
        let args = self.build_args();

        // Use toolchain from args, session default, or none
        let toolchain = self
            .toolchain
            .or_else(|| state.get_default_toolchain(None).unwrap_or(None));

        let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let cmd = create_cargo_command(&args_refs, toolchain.as_deref(), self.cargo_env.as_ref());
        execute_cargo_command(cmd, &project_path, "cargo run")
    }
}
