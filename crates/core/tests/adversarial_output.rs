use std::fs;
use std::path::Path;
use std::process::Command;

use ferret_core::{obfuscate, ObfuscationOptions};

#[test]
fn strong_output_resists_known_static_scanners() {
    let source = r#"
local outer_print = print
local _ENV = { print = outer_print, value = "secret_literal" }
local function read_value()
    return value
end
local function many()
    return read_value(), nil, "tail_secret"
end
local t = { "root_secret", many() }
print("adv", t[1], t[2], t[3] == nil, t[4])
"#;
    let result = obfuscate(
        source,
        ObfuscationOptions {
            seed: 909,
            ..ObfuscationOptions::default()
        },
    )
    .unwrap();
    assert!(static_findings(&result.code).is_empty());
    assert!(!simple_bytecode_tuple_locator(&result.code));
    assert!(old_helper_hook_targets(&result.code).is_empty());
    assert!(result.metadata.dump_resistance_level > 0);
    assert_eq!(result.metadata.adversarial_suite_version, 1);

    if lua_available() {
        let temp = tempfile::NamedTempFile::new().unwrap();
        fs::write(temp.path(), result.code).unwrap();
        let output = run_lua(temp.path());
        assert!(output.status.success());
        assert_eq!(
            output.stdout,
            b"adv\troot_secret\tsecret_literal\ttrue\ttail_secret\n"
        );
    }
}

#[test]
fn scanner_rejects_archived_weak_shape() {
    let weak = include_str!("../../../tests/adversarial/weak_output_sample.lua");
    assert!(!static_findings(weak).is_empty());
    assert!(simple_bytecode_tuple_locator(weak));
    assert!(!old_helper_hook_targets(weak).is_empty());
}

fn static_findings(code: &str) -> Vec<&'static str> {
    [
        "OP_",
        "LOADK",
        "CALLGLOBAL",
        "_f_",
        "ferret vm",
        "local cache",
        "if false then",
        "W[1],W[2],W[3],W[4]",
        "C[1][C[2]",
        "secret_literal",
        "tail_secret",
        "root_secret",
    ]
    .into_iter()
    .filter(|needle| code.contains(needle))
    .collect()
}

fn simple_bytecode_tuple_locator(code: &str) -> bool {
    code.contains("local O,A,B,C")
        || code.contains("for i=1,#W,4")
        || code.contains("O[j]=W[i]")
        || code.contains("W[1],W[2],W[3],W[4]")
}

fn old_helper_hook_targets(code: &str) -> Vec<&'static str> {
    ["_f_decode", "_f_pack", "_f_run", "function K"]
        .into_iter()
        .filter(|needle| code.contains(needle))
        .collect()
}

fn lua_available() -> bool {
    Command::new("lua").arg("-v").output().is_ok()
}

fn run_lua(path: &Path) -> std::process::Output {
    Command::new("lua").arg(path).output().unwrap()
}
