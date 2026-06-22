use std::fs;

use ferret_core::{obfuscate, ObfuscationOptions};

mod support;
use support::{lua_available, run_lua, run_lua_source};

#[test]
fn vm_preserves_lua_string_escapes() {
    if !lua_available() {
        return;
    }
    let source = r#"
local bytes = "\1\0\xff"
print("bytes", #bytes, string.byte(bytes, 1), string.byte(bytes, 2), string.byte(bytes, 3))
local zapped = "a\z
    b"
print("z", zapped, #zapped)
"#;
    let result = obfuscate(
        source,
        ObfuscationOptions {
            seed: 77,
            ..ObfuscationOptions::default()
        },
    )
    .unwrap();
    let temp = tempfile::NamedTempFile::new().unwrap();
    fs::write(temp.path(), result.code).unwrap();
    let original = run_lua_source(source);
    let obfuscated = run_lua(temp.path());
    assert_eq!(original.status.code(), obfuscated.status.code());
    assert_eq!(original.stdout, obfuscated.stdout);
}
