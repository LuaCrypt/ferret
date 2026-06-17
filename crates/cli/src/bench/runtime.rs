use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use anyhow::{bail, Context, Result};

use super::{BenchStats, RuntimeFileTiming};

pub(super) struct RuntimeBench {
    pub(super) lua: PathBuf,
    pub(super) runs: usize,
    pub(super) inner_runs: usize,
    pub(super) known_mismatches: BTreeSet<String>,
    pub(super) report_slowest: usize,
}

pub(super) fn bench_runtime(
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
        record_mismatch(original_path, runtime, stats);
        return Ok(());
    }

    let native = measure_lua_inner(
        &runtime.lua,
        original_path,
        runtime.runs,
        runtime.inner_runs,
    )?;
    let obfuscated =
        measure_lua_inner(&runtime.lua, temp.path(), runtime.runs, runtime.inner_runs)?;
    stats.runtime_files += 1;
    stats.runtime_native += native;
    stats.runtime_obfuscated += obfuscated;
    if runtime.report_slowest > 0 {
        stats.runtime_timings.push(RuntimeFileTiming {
            path: path_key(original_path),
            native,
            obfuscated,
        });
    }
    Ok(())
}

pub(super) fn check_lua(lua: &Path) -> Result<()> {
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

pub(super) fn load_known_mismatches(path: &Path) -> Result<BTreeSet<String>> {
    let contents =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    Ok(contents
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(normalize_path_key)
        .collect())
}

fn record_mismatch(original_path: &Path, runtime: &RuntimeBench, stats: &mut BenchStats) {
    stats.runtime_mismatches += 1;
    let key = path_key(original_path);
    if runtime.known_mismatches.contains(&key) {
        stats.runtime_known_mismatches += 1;
        eprintln!("runtime bench known mismatch: {}", original_path.display());
    } else {
        stats.runtime_unknown_mismatches += 1;
        eprintln!(
            "runtime bench unknown mismatch: {}",
            original_path.display()
        );
    }
}

fn run_lua(lua: &Path, path: &Path) -> Result<std::process::Output> {
    Command::new(lua)
        .arg(path)
        .output()
        .with_context(|| format!("failed to run {} {}", lua.display(), path.display()))
}

fn measure_lua_inner(lua: &Path, path: &Path, runs: usize, inner_runs: usize) -> Result<Duration> {
    let runner = tempfile::NamedTempFile::new().context("failed to create runtime runner")?;
    fs::write(runner.path(), runtime_runner()).context("failed to write runtime runner")?;
    let mut total = Duration::ZERO;
    for _ in 0..runs {
        let output = Command::new(lua)
            .arg(runner.path())
            .arg(path)
            .arg(inner_runs.to_string())
            .output()
            .with_context(|| format!("failed to run runtime timer for {}", path.display()))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!(
                "runtime measurement failed for {} with status {:?}: {}",
                path.display(),
                output.status.code(),
                stderr.trim()
            );
        }
        let stdout =
            String::from_utf8(output.stdout).context("runtime timer output was not utf8")?;
        let millis = stdout
            .trim()
            .parse::<f64>()
            .with_context(|| format!("runtime timer output was not a number: {stdout:?}"))?;
        total += Duration::from_secs_f64(millis / 1000.0);
    }
    Ok(total)
}

fn runtime_runner() -> &'static str {
    r#"
local path = assert(arg[1], "missing path")
local run_arg = assert(arg[2], "missing run count")
local runs = tonumber(run_arg)
local env = setmetatable({ print = function(...) end }, { __index = _ENV })
local fn, err = loadfile(path, "t", env)
if not fn then error(err, 0) end
local start = os.clock()
for _ = 1, runs do fn() end
io.write(string.format("%.9f", (os.clock() - start) * 1000))
"#
}

fn path_key(path: &Path) -> String {
    let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::new());
    let relative = path.strip_prefix(current_dir).unwrap_or(path);
    normalize_path_key(&relative.display().to_string())
}

fn normalize_path_key(path: &str) -> String {
    path.replace('\\', "/")
}
