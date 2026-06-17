use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Instant;

use anyhow::{Context, Result};
use ferret_core::{obfuscate, ObfuscationOptions, Preset};

use crate::args::BenchArgs;

#[derive(Default)]
struct BenchStats {
    files: usize,
    supported: usize,
    rejected: usize,
    failed: usize,
    input_bytes: usize,
    output_bytes: usize,
}

pub fn run_bench(args: BenchArgs) -> Result<()> {
    let preset = Preset::from_str(&args.preset)?;
    let seed = args.seed.unwrap_or(0xF3EE_2026);
    let paths = if args.paths.is_empty() {
        vec![PathBuf::from("tests/fixtures")]
    } else {
        args.paths
    };
    let mut files = Vec::new();
    for path in &paths {
        collect_lua(path, &mut files)?;
    }
    files.sort();
    let start = Instant::now();
    let mut stats = BenchStats::default();
    for path in files {
        bench_file(&path, seed, preset, &mut stats)?;
    }
    print_report(&paths, start.elapsed().as_secs_f64(), &stats);
    Ok(())
}

fn bench_file(path: &Path, seed: u64, preset: Preset, stats: &mut BenchStats) -> Result<()> {
    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    stats.files += 1;
    stats.input_bytes += bytes.len();
    let Ok(source) = std::str::from_utf8(&bytes) else {
        stats.rejected += 1;
        return Ok(());
    };
    match obfuscate(source, ObfuscationOptions { seed, preset }) {
        Ok(result) => {
            stats.supported += 1;
            stats.output_bytes += result.code.len();
        }
        Err(error) if is_expected_rejection(&error.to_string()) => stats.rejected += 1,
        Err(error) => {
            stats.failed += 1;
            eprintln!("bench failure: {}: {error}", path.display());
        }
    }
    Ok(())
}

fn collect_lua(path: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    if path.is_file() {
        if path.extension().and_then(|ext| ext.to_str()) == Some("lua") {
            files.push(path.to_path_buf());
        }
        return Ok(());
    }
    for entry in fs::read_dir(path).with_context(|| format!("failed to list {}", path.display()))? {
        let path = entry?.path();
        if path.is_dir() {
            collect_lua(&path, files)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("lua") {
            files.push(path);
        }
    }
    Ok(())
}

fn is_expected_rejection(error: &str) -> bool {
    error.contains("unsupported lua") || error.contains("parse error")
}

fn print_report(paths: &[PathBuf], seconds: f64, stats: &BenchStats) {
    let kib = stats.input_bytes as f64 / 1024.0;
    let rate = if seconds > 0.0 { kib / seconds } else { 0.0 };
    println!("Ferret OSS obfuscator bench");
    println!("paths: {}", display_paths(paths));
    println!("files: {}", stats.files);
    println!("supported: {}", stats.supported);
    println!("rejected: {}", stats.rejected);
    println!("failed: {}", stats.failed);
    println!("input_bytes: {}", stats.input_bytes);
    println!("output_bytes: {}", stats.output_bytes);
    println!("elapsed_ms: {:.3}", seconds * 1000.0);
    println!("throughput_kib_s: {:.2}", rate);
}

fn display_paths(paths: &[PathBuf]) -> String {
    paths
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ")
}
