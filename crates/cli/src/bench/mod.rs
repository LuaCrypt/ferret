use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use ferret_core::{obfuscate, ObfuscationOptions, Preset};

use crate::args::BenchArgs;

struct RuntimeBench {
    lua: PathBuf,
    runs: usize,
}

#[derive(Default)]
struct BenchStats {
    files: usize,
    supported: usize,
    rejected: usize,
    failed: usize,
    input_bytes: usize,
    output_bytes: usize,
    obfuscation_elapsed: Duration,
    runtime_files: usize,
    runtime_mismatches: usize,
    runtime_failed: usize,
    runtime_native: Duration,
    runtime_obfuscated: Duration,
}

pub fn run_bench(args: BenchArgs) -> Result<()> {
    let preset = Preset::from_str(&args.preset)?;
    let seed = args.seed.unwrap_or(0xF3EE_2026);
    let runtime = if args.runtime_overhead {
        if args.runtime_runs == 0 {
            bail!("--runtime-runs must be at least 1");
        }
        check_lua(&args.lua)?;
        Some(RuntimeBench {
            lua: args.lua.clone(),
            runs: args.runtime_runs,
        })
    } else {
        None
    };
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
        bench_file(&path, seed, preset, runtime.as_ref(), &mut stats)?;
    }
    print_report(
        &paths,
        start.elapsed().as_secs_f64(),
        runtime.as_ref(),
        &stats,
    );
    Ok(())
}

fn bench_file(
    path: &Path,
    seed: u64,
    preset: Preset,
    runtime: Option<&RuntimeBench>,
    stats: &mut BenchStats,
) -> Result<()> {
    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    stats.files += 1;
    stats.input_bytes += bytes.len();
    let Ok(source) = std::str::from_utf8(&bytes) else {
        stats.rejected += 1;
        return Ok(());
    };
    let obfuscation_start = Instant::now();
    let result = obfuscate(source, ObfuscationOptions { seed, preset });
    stats.obfuscation_elapsed += obfuscation_start.elapsed();
    match result {
        Ok(result) => {
            stats.supported += 1;
            stats.output_bytes += result.code.len();
            if let Some(runtime) = runtime {
                bench_runtime(path, &result.code, runtime, stats)?;
            }
        }
        Err(error) if is_expected_rejection(&error.to_string()) => stats.rejected += 1,
        Err(error) => {
            stats.failed += 1;
            eprintln!("bench failure: {}: {error}", path.display());
        }
    }
    Ok(())
}

fn bench_runtime(
    original_path: &Path,
    obfuscated_code: &str,
    runtime: &RuntimeBench,
    stats: &mut BenchStats,
) -> Result<()> {
    let temp = tempfile::NamedTempFile::new().context("failed to create runtime bench file")?;
    fs::write(temp.path(), obfuscated_code).context("failed to write runtime bench file")?;

    let original = run_lua(&runtime.lua, original_path)
        .with_context(|| format!("failed to run native lua for {}", original_path.display()))?;
    let obfuscated = run_lua(&runtime.lua, temp.path()).with_context(|| {
        format!(
            "failed to run obfuscated lua for {}",
            original_path.display()
        )
    })?;
    if !original.status.success() || !obfuscated.status.success() {
        stats.runtime_failed += 1;
        eprintln!(
            "runtime bench failure: {}: native_status={:?} obfuscated_status={:?}",
            original_path.display(),
            original.status.code(),
            obfuscated.status.code()
        );
        return Ok(());
    }
    if original.stdout != obfuscated.stdout {
        stats.runtime_mismatches += 1;
        eprintln!("runtime bench mismatch: {}", original_path.display());
        return Ok(());
    }

    let native = measure_lua(&runtime.lua, original_path, runtime.runs)?;
    let obfuscated = measure_lua(&runtime.lua, temp.path(), runtime.runs)?;
    stats.runtime_files += 1;
    stats.runtime_native += native;
    stats.runtime_obfuscated += obfuscated;
    Ok(())
}

fn check_lua(lua: &Path) -> Result<()> {
    let output = Command::new(lua)
        .arg("-v")
        .output()
        .with_context(|| format!("failed to run lua executable {}", lua.display()))?;
    if !output.status.success() {
        bail!(
            "lua executable {} exited with status {:?}",
            lua.display(),
            output.status.code()
        );
    }
    Ok(())
}

fn run_lua(lua: &Path, path: &Path) -> Result<std::process::Output> {
    Command::new(lua)
        .arg(path)
        .output()
        .with_context(|| format!("failed to run {} {}", lua.display(), path.display()))
}

fn measure_lua(lua: &Path, path: &Path, runs: usize) -> Result<Duration> {
    let start = Instant::now();
    for _ in 0..runs {
        let output = run_lua(lua, path)?;
        if !output.status.success() {
            bail!(
                "runtime measurement failed for {} with status {:?}",
                path.display(),
                output.status.code()
            );
        }
    }
    Ok(start.elapsed())
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

fn print_report(
    paths: &[PathBuf],
    seconds: f64,
    runtime: Option<&RuntimeBench>,
    stats: &BenchStats,
) {
    let kib = stats.input_bytes as f64 / 1024.0;
    let obfuscation_seconds = stats.obfuscation_elapsed.as_secs_f64();
    let rate = if obfuscation_seconds > 0.0 {
        kib / obfuscation_seconds
    } else {
        0.0
    };
    let size_overhead = if stats.input_bytes > 0 {
        stats.output_bytes as f64 / stats.input_bytes as f64
    } else {
        0.0
    };
    println!("Ferret OSS obfuscator bench");
    println!("paths: {}", display_paths(paths));
    println!("files: {}", stats.files);
    println!("supported: {}", stats.supported);
    println!("rejected: {}", stats.rejected);
    println!("failed: {}", stats.failed);
    println!("input_bytes: {}", stats.input_bytes);
    println!("output_bytes: {}", stats.output_bytes);
    println!("size_overhead_x: {:.2}", size_overhead);
    println!("elapsed_ms: {:.3}", seconds * 1000.0);
    println!(
        "obfuscation_elapsed_ms: {:.3}",
        stats.obfuscation_elapsed.as_secs_f64() * 1000.0
    );
    println!("throughput_kib_s: {:.2}", rate);
    if let Some(runtime) = runtime {
        let native_ms = stats.runtime_native.as_secs_f64() * 1000.0;
        let obfuscated_ms = stats.runtime_obfuscated.as_secs_f64() * 1000.0;
        let runtime_overhead = if native_ms > 0.0 {
            obfuscated_ms / native_ms
        } else {
            0.0
        };
        println!("runtime_files: {}", stats.runtime_files);
        println!("runtime_mismatches: {}", stats.runtime_mismatches);
        println!("runtime_failed: {}", stats.runtime_failed);
        println!("runtime_runs_per_file: {}", runtime.runs);
        println!("runtime_native_ms: {:.3}", native_ms);
        println!("runtime_obfuscated_ms: {:.3}", obfuscated_ms);
        println!("runtime_overhead_x: {:.2}", runtime_overhead);
    }
}

fn display_paths(paths: &[PathBuf]) -> String {
    paths
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ")
}
