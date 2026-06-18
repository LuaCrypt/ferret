#![allow(dead_code)]

use std::fs;
use std::path::Path;
use std::process::Command;

const STATIC_NEEDLES: &[&str] = &[
    "OP_",
    "LOADK",
    "CALLGLOBAL",
    "_f_",
    "ferret vm",
    "local cache",
    "if false then",
    "while true do",
    "W[1],W[2],W[3],W[4]",
    "C[1][C[2]",
];

pub fn assert_hardened_output_shape(code: &str, source_literals: &[&str]) {
    let findings = static_findings(code, source_literals);
    assert!(findings.is_empty(), "{findings:?}");
}

pub fn static_findings(code: &str, extra_needles: &[&str]) -> Vec<String> {
    STATIC_NEEDLES
        .iter()
        .chain(extra_needles.iter())
        .filter(|needle| code.contains(**needle))
        .map(|needle| (*needle).to_string())
        .collect()
}

pub fn simple_bytecode_tuple_locator(code: &str) -> bool {
    code.contains("local O,A,B,C")
        || code.contains("for i=1,#W,4")
        || code.contains("O[j]=W[i]")
        || code.contains("W[1],W[2],W[3],W[4]")
}

pub fn old_helper_hook_targets(code: &str) -> Vec<&'static str> {
    ["_f_decode", "_f_pack", "_f_run", "function K"]
        .into_iter()
        .filter(|needle| code.contains(needle))
        .collect()
}

pub fn lua_available() -> bool {
    Command::new("lua").arg("-v").output().is_ok()
}

pub fn run_lua(path: &Path) -> std::process::Output {
    Command::new("lua").arg(path).output().unwrap()
}

pub fn run_lua_source(source: &str) -> std::process::Output {
    let temp = tempfile::NamedTempFile::new().unwrap();
    fs::write(temp.path(), source).unwrap();
    run_lua(temp.path())
}
