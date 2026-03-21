use crate::state::CargoTools;
use crate::tools::cargo_utils::{create_cargo_command, execute_cargo_command};
use anyhow::Result;
use mcplease::{
    traits::{Tool, WithExamples},
    types::Example,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Run cargo test to execute tests
#[derive(Debug, Default, Serialize, Deserialize, schemars::JsonSchema, clap::Args)]
#[serde(rename = "cargo_test")]
pub struct CargoTest {
    /// Optional package name to test (for workspaces)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub package: Option<String>,

    /// Optional specific test name to run
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub test_name: Option<String>,

    /// Don't capture stdout/stderr of tests, allow printing to console
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub no_capture: Option<bool>,

    /// Optional Rust toolchain to use (e.g., 'stable', 'nightly', '1.70.0')
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub toolchain: Option<String>,

    /// Optional environment variables to set for the cargo command
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(skip)]
    pub cargo_env: Option<HashMap<String, String>>,

    /// Use cargo-nextest instead of cargo test
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub use_nextest: Option<bool>,

    /// Additional cargo arguments passed before any `--` separator
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(skip)]
    pub extra_args: Option<Vec<String>>,
}

impl WithExamples for CargoTest {
    fn examples() -> Vec<Example<Self>> {
        vec![
            Example {
                description: "Run all tests in current project",
                item: Self::default(),
            },
            Example {
                description: "Run tests for a specific package",
                item: Self {
                    package: Some("my-lib".into()),
                    ..Self::default()
                },
            },
            Example {
                description: "Run a specific test",
                item: Self {
                    test_name: Some("test_addition".into()),
                    ..Self::default()
                },
            },
            Example {
                description: "Run tests with no capture (show println! output)",
                item: Self {
                    no_capture: Some(true),
                    ..Self::default()
                },
            },
            Example {
                description: "Run tests with custom environment",
                item: Self {
                    cargo_env: Some(
                        [
                            ("RUST_LOG".into(), "debug".into()),
                            ("TEST_ENV".into(), "true".into()),
                        ]
                        .into(),
                    ),
                    ..Self::default()
                },
            },
            Example {
                description: "Run tests using cargo-nextest",
                item: Self {
                    use_nextest: Some(true),
                    ..Self::default()
                },
            },
        ]
    }
}

impl CargoTest {
    /// Build the cargo test argument list.
    pub fn build_args(&self) -> Vec<String> {
        let nextest = self.use_nextest.unwrap_or(false);

        let mut args = if nextest {
            vec!["nextest".to_string(), "run".to_string()]
        } else {
            vec!["test".to_string()]
        };

        if let Some(ref package) = self.package {
            args.push("--package".to_string());
            args.push(package.clone());
        }

        if let Some(ref test_name) = self.test_name {
            args.push(test_name.clone());
        }

        if let Some(ref extra) = self.extra_args {
            args.extend(extra.iter().cloned());
        }

        if self.no_capture.unwrap_or(false) {
            if nextest {
                args.push("--no-capture".to_string());
            } else {
                args.push("--".to_string());
                args.push("--nocapture".to_string());
            }
        }

        args
    }
}

impl Tool<CargoTools> for CargoTest {
    fn execute(self, state: &mut CargoTools) -> Result<String> {
        let project_path = state.ensure_rust_project(None)?;

        let args = self.build_args();
        let nextest = self.use_nextest.unwrap_or(false);

        // Use toolchain from args, session default, or none
        let toolchain = self
            .toolchain
            .or_else(|| state.get_default_toolchain(None).unwrap_or(None));

        let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let cmd = create_cargo_command(&args_refs, toolchain.as_deref(), self.cargo_env.as_ref());
        let label = if nextest {
            "cargo nextest run"
        } else {
            "cargo test"
        };
        execute_cargo_command(cmd, &project_path, label)
    }
}
