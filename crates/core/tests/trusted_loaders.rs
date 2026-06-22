use std::fs;

use ferret_core::{obfuscate, ObfuscationOptions};

mod support;
use support::{lua_available, run_lua, run_lua_source};

#[test]
fn trusted_mode_allows_dynamic_loaders_without_debug() {
    let strict = obfuscate("load('return 1')()", ObfuscationOptions::default()).unwrap_err();
    assert!(strict.to_string().contains("load"));

    let debug = obfuscate(
        "return require('debug')",
        ObfuscationOptions {
            allow_dynamic_loaders: true,
            ..ObfuscationOptions::default()
        },
    )
    .unwrap_err();
    assert!(debug.to_string().contains("debug"));

    if !lua_available() {
        return;
    }
    let source = r#"
local f = assert(load("return 40 + 2"))
print("loaded", f())
"#;
    let result = obfuscate(
        source,
        ObfuscationOptions {
            seed: 77,
            allow_dynamic_loaders: true,
            ..ObfuscationOptions::default()
        },
    )
    .unwrap();
    assert!(result.metadata.dynamic_loaders_allowed);
    assert!(!result.metadata.vm_only);
    let temp = tempfile::NamedTempFile::new().unwrap();
    fs::write(temp.path(), result.code).unwrap();
    let original = run_lua_source(source);
    let obfuscated = run_lua(temp.path());
    assert_eq!(original.status.code(), obfuscated.status.code());
    assert_eq!(original.stdout, obfuscated.stdout);
}
