use std::fs;

use ferret_core::{obfuscate, ObfuscationOptions};

mod support;
use support::{lua_available, run_lua, run_lua_source};

#[test]
fn vm_preserves_general_vararg_shapes() {
    if !lua_available() {
        return;
    }
    let source = r#"
local function outer(...)
    local head = ...
    local a, b, c = ...
    local t = { "base", ... }
    local function emit(...)
        print("call", ...)
    end
    local function two(...)
        return ...
    end
    emit("var", ...)
    local x, y = two(...)
    print("scalar", head)
    print("fixed", a, b, c)
    print("table", #t, t[1], t[2], t[3], t[4])
    print("multi", x, y)
end
outer("alpha", "beta", "gamma")
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
