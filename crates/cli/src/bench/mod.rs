use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use ferret_core::{obfuscate, ObfuscationOptions, Preset};

use crate::args::BenchArgs;

mod runtime;

use runtime::{bench_runtime, check_lua, load_known_mismatches, RuntimeBench};

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
    runtime_known_mismatches: usize,
    runtime_unknown_mismatches: usize,
    runtime_mismatches: usize,
    runtime_failed: usize,
    runtime_native: Duration,
    runtime_obfuscated: Duration,
    runtime_timings: Vec<RuntimeFileTiming>,
}

#[derive(Clone)]
struct RuntimeFileTiming {
    path: String,
    native: Duration,
    obfuscated: Duration,
}

pub fn run_bench(args: BenchArgs) -> Result<()> {
    let preset = Preset::from_str(&args.preset)?;
    let seed = args.seed.unwrap_or(0xF3EE_2026);
    let runtime = if args.runtime_overhead {
        if args.runtime_runs == 0 {
            bail!("--runtime-runs must be at least 1");
        }
        if args.runtime_inner_runs == 0 {
            bail!("--runtime-inner-runs must be at least 1");
        }
        check_lua(&args.lua)?;
        let known_mismatches = if let Some(path) = &args.runtime_known_mismatches {
            load_known_mismatches(path)?
        } else {
            BTreeSet::new()
        };
        Some(RuntimeBench {
            lua: args.lua.clone(),
            runs: args.runtime_runs,
            inner_runs: args.runtime_inner_runs,
            known_mismatches,
            report_slowest: args.runtime_report_slowest,
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
    if stats.runtime_unknown_mismatches > 0 {
        bail!(
            "runtime bench found {} unknown stdout mismatch(es)",
            stats.runtime_unknown_mismatches
        );
    }
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
        println!(
            "runtime_known_mismatches: {}",
            stats.runtime_known_mismatches
        );
        println!(
            "runtime_unknown_mismatches: {}",
            stats.runtime_unknown_mismatches
        );
        println!("runtime_failed: {}", stats.runtime_failed);
        println!("runtime_runs_per_file: {}", runtime.runs);
        println!("runtime_inner_runs: {}", runtime.inner_runs);
        println!("runtime_native_ms: {:.3}", native_ms);
        println!("runtime_obfuscated_ms: {:.3}", obfuscated_ms);
        println!("runtime_overhead_x: {:.2}", runtime_overhead);
        if runtime.report_slowest > 0 {
            print_slowest_runtime_files(&stats.runtime_timings, runtime.report_slowest);
        }
    }
}

fn print_slowest_runtime_files(timings: &[RuntimeFileTiming], limit: usize) {
    let mut timings = timings.to_vec();
    timings.sort_by_key(|timing| std::cmp::Reverse(timing.obfuscated));
    for timing in timings.into_iter().take(limit) {
        let native_ms = timing.native.as_secs_f64() * 1000.0;
        let obfuscated_ms = timing.obfuscated.as_secs_f64() * 1000.0;
        let overhead = if native_ms > 0.0 {
            obfuscated_ms / native_ms
        } else {
            0.0
        };
        println!(
            "runtime_slowest_file: {} native_ms={:.3} obfuscated_ms={:.3} overhead_x={:.2}",
            timing.path, native_ms, obfuscated_ms, overhead
        );
    }
}

fn display_paths(paths: &[PathBuf]) -> String {
    paths
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ")
}
