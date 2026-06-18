use std::fs;
use std::process::Command;

#[test]
fn cli_writes_deterministic_vm_output_and_metadata() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("in.lua");
    let out = temp.path().join("out.lua");
    let meta = temp.path().join("meta.json");
    fs::write(&input, "local x = 4 * 2\nprint('cli', x)\n").unwrap();
    let status = Command::new(env!("CARGO_BIN_EXE_ferret"))
        .args([
            "obfuscate",
            input.to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
            "--seed",
            "11",
            "--metadata",
            meta.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success());
    let code = fs::read_to_string(out).unwrap();
    let metadata = fs::read_to_string(meta).unwrap();
    assert!(!code.contains("load("));
    assert!(metadata.contains("\"vm_only\": true"));
    assert!(metadata.contains("\"runtime_names_obfuscated\": true"));
    assert!(metadata.contains("\"output_hardened\": true"));
    assert!(metadata.contains("\"fake_opcode_count\":"));
    assert!(metadata.contains("\"fake_bytecode_words\":"));
    assert!(metadata.contains("\"bytecode_integrity_tag\":"));
    assert!(metadata.contains("\"runtime_template_variant\":"));
    assert!(metadata.contains("\"bytecode_layout_variant\":"));
    assert!(metadata.contains("\"constant_layout_variant\":"));
    assert!(metadata.contains("\"semantic_alias_count\":"));
    assert!(metadata.contains("\"handler_polymorphism_level\":"));
    assert!(metadata.contains("\"output_hardening_level\":"));
    assert!(metadata.contains("\"runtime_integrity_checks\": true"));
    assert!(metadata.contains("\"delayed_string_constants\": true"));
}

#[test]
fn cli_bench_reports_fixture_summary() {
    let fixtures = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("tests/fixtures");
    let output = Command::new(env!("CARGO_BIN_EXE_ferret"))
        .args(["bench", fixtures.to_str().unwrap(), "--seed", "11"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Ferret OSS obfuscator bench"));
    assert!(stdout.contains("supported: 3"));
}

#[test]
fn cli_bench_reports_runtime_overhead() {
    if Command::new("lua").arg("-v").output().is_err() {
        return;
    }
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("tests/fixtures/control.lua");
    let output = Command::new(env!("CARGO_BIN_EXE_ferret"))
        .args([
            "bench",
            "--runtime-overhead",
            "--runtime-runs",
            "1",
            fixture.to_str().unwrap(),
            "--seed",
            "11",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("runtime_files: 1"));
    assert!(stdout.contains("runtime_overhead_x:"));
}
