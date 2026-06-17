mod args;
mod bench;

use std::fs;
use std::str::FromStr;

use anyhow::{Context, Result};
use args::{Cli, Command};
use bench::run_bench;
use clap::Parser;
use ferret_core::{obfuscate, ObfuscationOptions, Preset};

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Obfuscate(args) => run_obfuscate(args)?,
        Command::Bench(args) => run_bench(args)?,
    }
    Ok(())
}

fn run_obfuscate(args: args::ObfuscateArgs) -> Result<()> {
    let source = fs::read_to_string(&args.input)
        .with_context(|| format!("failed to read {}", args.input.display()))?;
    let seed = args.seed.unwrap_or(0xF3EE_2026);
    let preset = Preset::from_str(&args.preset)?;
    let result = obfuscate(&source, ObfuscationOptions { seed, preset })?;
    if let Some(parent) = args.output.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    fs::write(&args.output, result.code)
        .with_context(|| format!("failed to write {}", args.output.display()))?;
    if let Some(path) = args.metadata {
        let json = serde_json::to_string_pretty(&result.metadata)?;
        fs::write(&path, json).with_context(|| format!("failed to write {}", path.display()))?;
    }
    Ok(())
}
