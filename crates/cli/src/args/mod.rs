use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "ferret")]
#[command(about = "A strong OSS Lua VM obfuscator")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Obfuscate(ObfuscateArgs),
    Bench(BenchArgs),
}

#[derive(Debug, Args)]
pub struct ObfuscateArgs {
    pub input: PathBuf,

    #[arg(short, long)]
    pub output: PathBuf,

    #[arg(long)]
    pub seed: Option<u64>,

    #[arg(long, default_value = "strong")]
    pub preset: String,

    #[arg(long)]
    pub metadata: Option<PathBuf>,

    #[arg(long)]
    pub allow_dynamic_loaders: bool,
}

#[derive(Debug, Args)]
pub struct BenchArgs {
    pub paths: Vec<PathBuf>,

    #[arg(long)]
    pub seed: Option<u64>,

    #[arg(long, default_value = "strong")]
    pub preset: String,

    #[arg(long)]
    pub runtime_overhead: bool,

    #[arg(long, default_value_t = 3)]
    pub runtime_runs: usize,

    #[arg(long, default_value_t = 1)]
    pub runtime_inner_runs: usize,

    #[arg(long)]
    pub runtime_known_mismatches: Option<PathBuf>,

    #[arg(long, default_value_t = 0)]
    pub runtime_report_slowest: usize,

    #[arg(long, default_value = "lua")]
    pub lua: PathBuf,

    #[arg(long)]
    pub allow_dynamic_loaders: bool,
}
