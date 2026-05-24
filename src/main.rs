mod state;
mod tools;

#[cfg(test)]
mod tests;

use anyhow::Result;
use mcplease::server_info;
use state::CargoTools;

const INSTRUCTIONS: &str = "Cargo operations for Rust projects.

Use set_working_directory to set the project directory first, then run cargo commands.";

fn main() -> Result<()> {
    // cargo subcommand convention: `cargo stevedore serve` invokes
    // `cargo-stevedore stevedore serve` — strip the redundant arg.
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).is_some_and(|s| s == "stevedore") {
        let status = std::process::Command::new(&args[0])
            .args(&args[2..])
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .status()?;
        std::process::exit(status.code().unwrap_or(1));
    }

    let mut state = CargoTools::new()?;
    mcplease::run::<tools::Tools, _>(&mut state, server_info!(), Some(INSTRUCTIONS))
}
