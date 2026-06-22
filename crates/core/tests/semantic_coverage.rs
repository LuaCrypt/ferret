use std::fs;
use std::path::{Path, PathBuf};

use ferret_core::{obfuscate, ObfuscationOptions};

mod support;
use support::{lua_available, run_lua};

const MIN_PERCENT: usize = 90;

#[test]
fn lua54_semantic_coverage_excluding_debug_lib_reaches_90_percent() {
    if !lua_available() {
        return;
    }

    let mut files = Vec::new();
    collect_lua(&repo_path("tests/scripts"), &mut files);
    collect_lua(&repo_path("tests/lua54_conformance"), &mut files);
    files.sort();

    let mut total = 0usize;
    let mut passed = 0usize;
    let mut gaps = Vec::new();

    for path in files {
        let source = fs::read_to_string(&path).unwrap();
        if uses_debug_lib(&source) {
            continue;
        }
        total += 1;
        match semantic_match(&path, &source) {
            Ok(true) => passed += 1,
            Ok(false) => gaps.push(format!("runtime mismatch: {}", relative(&path))),
            Err(error) => gaps.push(format!("unsupported: {}: {error}", relative(&path))),
        }
    }

    assert!(total > 0);
    assert!(
        passed * 100 >= total * MIN_PERCENT,
        "Lua 5.4 semantic coverage excluding debug lib is {passed}/{total} ({:.2}%), below {MIN_PERCENT}%\nfirst gaps:\n{}",
        coverage_percent(passed, total),
        gaps.iter().take(20).cloned().collect::<Vec<_>>().join("\n")
    );
}

fn semantic_match(path: &Path, source: &str) -> Result<bool, String> {
    let result = obfuscate(
        source,
        ObfuscationOptions {
            seed: 77,
            ..ObfuscationOptions::default()
        },
    )
    .map_err(|error| error.to_string())?;
    let temp = tempfile::NamedTempFile::new().map_err(|error| error.to_string())?;
    fs::write(temp.path(), result.code).map_err(|error| error.to_string())?;

    let original = run_lua(path);
    let obfuscated = run_lua(temp.path());
    Ok(original.status.success()
        && obfuscated.status.success()
        && original.stdout == obfuscated.stdout)
}

fn collect_lua(path: &Path, files: &mut Vec<PathBuf>) {
    if path.is_file() {
        if path.extension().and_then(|ext| ext.to_str()) == Some("lua") {
            files.push(path.to_path_buf());
        }
        return;
    }
    for entry in fs::read_dir(path).unwrap() {
        collect_lua(&entry.unwrap().path(), files);
    }
}

fn uses_debug_lib(source: &str) -> bool {
    source
        .split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
        .any(|word| word == "debug")
}

fn coverage_percent(passed: usize, total: usize) -> f64 {
    passed as f64 * 100.0 / total as f64
}

fn repo_path(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn relative(path: &Path) -> String {
    path.strip_prefix(repo_path(""))
        .unwrap_or(path)
        .display()
        .to_string()
}
