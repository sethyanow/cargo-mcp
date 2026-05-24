use anyhow::Result;
use std::{
    collections::HashMap,
    io::Read as _,
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

/// Helper to create a cargo command with optional toolchain and environment variables
pub fn create_cargo_command(
    cargo_args: &[&str],
    toolchain: Option<&str>,
    env_vars: Option<&HashMap<String, String>>,
) -> Command {
    let mut cmd = if let Some(toolchain) = toolchain {
        let mut cmd = Command::new("rustup");
        cmd.args(["run", toolchain, "cargo"]);
        cmd.args(cargo_args);
        cmd
    } else {
        let mut cmd = Command::new("cargo");
        cmd.args(cargo_args);
        cmd
    };

    // Apply environment variables if provided
    if let Some(env_map) = env_vars {
        for (key, value) in env_map {
            cmd.env(key, value);
        }
    }

    cmd
}

const CARGO_COMMAND_TIMEOUT_SECS: u64 = 600;

/// Execute a cargo command and format the output for MCP response
pub fn execute_cargo_command(
    cmd: Command,
    project_path: &PathBuf,
    command_name: &str,
) -> Result<String> {
    execute_cargo_command_with_timeout(
        cmd,
        project_path,
        command_name,
        Duration::from_secs(CARGO_COMMAND_TIMEOUT_SECS),
    )
}

/// Format a command for display
fn format_command(cmd: &Command) -> String {
    let program = cmd.get_program().to_string_lossy();
    let args = cmd
        .get_args()
        .map(|arg| shell_escape(&arg.to_string_lossy()))
        .collect::<Vec<_>>()
        .join(" ");

    if args.is_empty() {
        program.to_string()
    } else {
        format!("{program} {args}")
    }
}

/// Simple shell escaping for display purposes
fn shell_escape(arg: &str) -> String {
    if arg.contains(' ') || arg.contains('"') || arg.contains('\'') || arg.contains('\\') {
        format!("{arg:?}") // Uses Rust's debug escaping
    } else {
        arg.to_string()
    }
}

fn execute_cargo_command_with_timeout(
    mut cmd: Command,
    project_path: &PathBuf,
    command_name: &str,
    timeout: Duration,
) -> Result<String> {
    cmd.current_dir(project_path);

    let mut child = cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;

    let stdout_pipe = child.stdout.take().unwrap();
    let stderr_pipe = child.stderr.take().unwrap();

    let stdout_thread = thread::spawn(move || {
        let mut buf = Vec::new();
        let mut r = stdout_pipe;
        r.read_to_end(&mut buf).map(|_| buf)
    });
    let stderr_thread = thread::spawn(move || {
        let mut buf = Vec::new();
        let mut r = stderr_pipe;
        r.read_to_end(&mut buf).map(|_| buf)
    });

    let timeout_secs = timeout.as_secs();
    let deadline = Instant::now() + timeout;
    let status = loop {
        match child.try_wait()? {
            Some(status) => break status,
            None if Instant::now() >= deadline => {
                let _ = child.kill();
                child.wait()?;
                anyhow::bail!("cargo {command_name} timed out after {timeout_secs}s");
            }
            None => thread::sleep(Duration::from_millis(100)),
        }
    };

    let stdout_bytes = stdout_thread.join().unwrap()?;
    let stderr_bytes = stderr_thread.join().unwrap()?;
    let stdout = String::from_utf8_lossy(&stdout_bytes);
    let stderr = String::from_utf8_lossy(&stderr_bytes);

    let mut result = format!("=== {command_name} ===\n");
    result.push_str(&format!(
        "📁 Working directory: {}\n",
        project_path.display()
    ));
    result.push_str(&format!("🔧 Command: {}\n\n", format_command(&cmd)));

    if status.success() {
        result.push_str("✅ Command completed successfully\n\n");
    } else {
        result.push_str(&format!(
            "❌ Command failed with exit code: {}\n\n",
            status.code().unwrap_or(-1)
        ));
    }

    if !stdout.is_empty() {
        result.push_str("📤 STDOUT:\n");
        result.push_str(&stdout);
        if !stdout.ends_with('\n') {
            result.push('\n');
        }
        result.push('\n');
    }

    if !stderr.is_empty() {
        result.push_str("📤 STDERR:\n");
        result.push_str(&stderr);
        if !stderr.ends_with('\n') {
            result.push('\n');
        }
        result.push('\n');
    }

    if stdout.is_empty() && stderr.is_empty() {
        result.push_str("ℹ️  No output produced\n");
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn execute_cargo_command_times_out() {
        let mut cmd = Command::new("sleep");
        cmd.arg("60");
        let result = execute_cargo_command_with_timeout(
            cmd,
            &PathBuf::from("/tmp"),
            "sleep-test",
            Duration::from_secs(1),
        );
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("timed out"), "expected timeout, got: {msg}");
        assert!(
            msg.contains("sleep-test"),
            "expected command name, got: {msg}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn execute_cargo_command_normal_completion() {
        let mut cmd = Command::new("echo");
        cmd.arg("hello");
        let result = execute_cargo_command_with_timeout(
            cmd,
            &PathBuf::from("/tmp"),
            "echo-test",
            Duration::from_secs(10),
        );
        let output = result.expect("should succeed");
        assert!(output.contains("hello"), "expected stdout, got: {output}");
        assert!(
            output.contains("Command completed successfully"),
            "expected success marker, got: {output}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn execute_cargo_command_failed_exit() {
        let cmd = Command::new("false");
        let result = execute_cargo_command_with_timeout(
            cmd,
            &PathBuf::from("/tmp"),
            "false-test",
            Duration::from_secs(10),
        );
        let output = result.expect("non-zero exit is Ok, not Err");
        assert!(
            output.contains("Command failed with exit code"),
            "expected failure marker, got: {output}"
        );
    }

    #[test]
    fn execute_cargo_command_spawn_failure() {
        let cmd = Command::new("nonexistent_binary_xyz_12345");
        let result = execute_cargo_command_with_timeout(
            cmd,
            &PathBuf::from("/tmp"),
            "bad-binary",
            Duration::from_secs(10),
        );
        assert!(result.is_err(), "nonexistent binary should fail to spawn");
    }

    #[cfg(unix)]
    #[test]
    fn execute_cargo_command_immediate_exit() {
        let cmd = Command::new("true");
        let result = execute_cargo_command_with_timeout(
            cmd,
            &PathBuf::from("/tmp"),
            "true-test",
            Duration::from_secs(10),
        );
        let output = result.expect("immediate exit should succeed");
        assert!(
            output.contains("Command completed successfully"),
            "expected success, got: {output}"
        );
    }
}
