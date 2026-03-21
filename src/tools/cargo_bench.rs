use crate::state::CargoTools;
use crate::tools::cargo_utils::{create_cargo_command, execute_cargo_command};
use anyhow::Result;
use mcplease::{
    traits::{Tool, WithExamples},
    types::Example,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Run cargo bench to execute benchmarks
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema, clap::Args)]
#[serde(rename = "cargo_bench")]
pub struct CargoBench {
    /// Optional package name to benchmark (for workspaces)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub package: Option<String>,

    /// Optional specific benchmark name to run
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub bench_name: Option<String>,

    /// Optional baseline name for comparison
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub baseline: Option<String>,

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

impl WithExamples for CargoBench {
    fn examples() -> Vec<Example<Self>> {
        vec![
            Example {
                description: "Run all benchmarks",
                item: Self {
                    package: None,
                    bench_name: None,
                    baseline: None,
                    toolchain: None,
                    cargo_env: None,
                    extra_args: None,
                },
            },
            Example {
                description: "Run a specific benchmark",
                item: Self {
                    package: None,
                    bench_name: Some("my_benchmark".into()),
                    baseline: None,
                    toolchain: None,
                    cargo_env: None,
                    extra_args: None,
                },
            },
            Example {
                description: "Run benchmarks for a specific package",
                item: Self {
                    package: Some("my-lib".into()),
                    bench_name: None,
                    baseline: None,
                    toolchain: None,
                    cargo_env: None,
                    extra_args: None,
                },
            },
            Example {
                description: "Run benchmarks with a baseline for comparison",
                item: Self {
                    package: None,
                    bench_name: None,
                    baseline: Some("main".into()),
                    toolchain: None,
                    cargo_env: None,
                    extra_args: None,
                },
            },
        ]
    }
}

impl CargoBench {
    /// Build the cargo bench argument list.
    pub fn build_args(&self) -> Vec<String> {
        let mut args = vec!["bench".to_string()];

        if let Some(ref package) = self.package {
            args.push("--package".to_string());
            args.push(package.clone());
        }

        if let Some(ref bench_name) = self.bench_name {
            args.push(bench_name.clone());
        }

        if let Some(ref extra) = self.extra_args {
            args.extend(extra.iter().cloned());
        }

        if let Some(ref baseline) = self.baseline {
            args.push("--".to_string());
            args.push("--save-baseline".to_string());
            args.push(baseline.clone());
        }

        args
    }
}

impl Tool<CargoTools> for CargoBench {
    fn execute(self, state: &mut CargoTools) -> Result<String> {
        let project_path = state.ensure_rust_project(None)?;
        let args = self.build_args();

        // Use toolchain from args, session default, or none
        let toolchain = self.toolchain
            .or_else(|| state.get_default_toolchain(None).unwrap_or(None));

        let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let cmd = create_cargo_command(&args_refs, toolchain.as_deref(), self.cargo_env.as_ref());
        execute_cargo_command(cmd, &project_path, "cargo bench")
    }
}
