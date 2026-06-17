use std::fs;
use std::path::Path;
use std::process::Command;

use ferret_core::{obfuscate, ObfuscationOptions};

#[test]
fn vm_matches_supported_fixtures() {
    if !lua_available() {
        return;
    }
    for path in supported_paths() {
        let source = fs::read_to_string(&path).unwrap();
        let result = obfuscate(
            &source,
            ObfuscationOptions {
                seed: 77,
                ..ObfuscationOptions::default()
            },
        )
        .unwrap();
        assert!(result.metadata.vm_only);
        assert!(result.metadata.encrypted_bytecode);
        assert!(result.metadata.encrypted_constants);
        assert!(result.metadata.custom_opcodes);
        assert!(!result.metadata.source_reconstruction);
        assert!(!result.code.contains("load("));

        let temp = tempfile::NamedTempFile::new().unwrap();
        fs::write(temp.path(), result.code).unwrap();
        let original = run_lua(&path);
        let obfuscated = run_lua(temp.path());
        let label = path.display().to_string();
        assert_eq!(original.status.code(), obfuscated.status.code(), "{label}");
        assert_eq!(original.stdout, obfuscated.stdout, "{label}");
    }
}

#[test]
fn deterministic_with_fixed_seed() {
    let source = fs::read_to_string(fixture_path("basic.lua")).unwrap();
    let first = obfuscate(
        &source,
        ObfuscationOptions {
            seed: 5,
            ..ObfuscationOptions::default()
        },
    )
    .unwrap();
    let second = obfuscate(
        &source,
        ObfuscationOptions {
            seed: 5,
            ..ObfuscationOptions::default()
        },
    )
    .unwrap();
    assert_eq!(first.code, second.code);
    assert_eq!(first.metadata.output_hash, second.metadata.output_hash);
    assert!(!first.code.contains("local W="));
    assert!(!first.code.contains("local C="));
    assert!(!first.code.contains("function dwv"));
    assert!(first.code.contains("C[1][C[2][k]]"));
    assert!(first.code.contains("local cache=C[3]"));

    let third = obfuscate(
        &source,
        ObfuscationOptions {
            seed: 6,
            ..ObfuscationOptions::default()
        },
    )
    .unwrap();
    assert_ne!(first.code, third.code);
}

#[test]
fn rejects_dynamic_loader() {
    let err = obfuscate("load('print(1)')()", ObfuscationOptions::default()).unwrap_err();
    assert!(err.to_string().contains("load"));
}

fn fixture_path(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("tests/fixtures")
        .join(name)
}

fn script_path(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("tests/scripts")
        .join(name)
}

fn supported_paths() -> Vec<std::path::PathBuf> {
    vec![
        fixture_path("basic.lua"),
        fixture_path("control.lua"),
        fixture_path("table.lua"),
        script_path("basics/002_basics.lua"),
        script_path("control_flow/002_control_flow.lua"),
        script_path("control_flow/004_control_flow.lua"),
        script_path("closures_upvalues/001_closures_upvalues.lua"),
        script_path("functions/002_functions.lua"),
        script_path("functions/003_functions.lua"),
        script_path("generic_for/001_generic_for.lua"),
        script_path("generic_for/002_generic_for.lua"),
        script_path("goto_labels/001_goto_labels.lua"),
        script_path("regressions/001_regressions.lua"),
        script_path("varargs_multireturn/001_varargs_multireturn.lua"),
        script_path("varargs_multireturn/009_varargs_multireturn.lua"),
        lua54_path("001_multireturn_assignment.lua"),
        lua54_path("005_goto_forward_backward.lua"),
        lua54_path("010_vararg_return_tail.lua"),
    ]
}

fn lua54_path(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("tests/lua54_conformance")
        .join(name)
}

fn lua_available() -> bool {
    Command::new("lua").arg("-v").output().is_ok()
}

fn run_lua(path: &Path) -> std::process::Output {
    Command::new("lua").arg(path).output().unwrap()
}
