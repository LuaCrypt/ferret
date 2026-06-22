use std::fs;

use ferret_core::{obfuscate, ObfuscationOptions};

mod support;
use support::{lua_available, run_lua, run_lua_source};

#[test]
fn vm_expands_trailing_call_arguments() {
    if !lua_available() {
        return;
    }
    let source = r##"
local function call(f, args)
    return f(table.unpack(args, 1, args.n))
end
local function collect(...)
    return select("#", ...), ...
end
local n, a, b, c = call(collect, {"a", nil, "c", n = 3})
print("open", n, a, b == nil, c)
"##;
    let result = obfuscate(
        source,
        ObfuscationOptions {
            seed: 77,
            allow_dynamic_loaders: true,
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
