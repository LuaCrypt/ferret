use std::fs;

use ferret_core::{obfuscate, ObfuscationOptions};

mod support;
use support::static_findings;
use support::{lua_available, old_helper_hook_targets, run_lua, simple_bytecode_tuple_locator};

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
    assert!(static_findings(&result.code, ADVERSARIAL_LITERALS).is_empty());
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
    assert!(!static_findings(weak, ADVERSARIAL_LITERALS).is_empty());
    assert!(simple_bytecode_tuple_locator(weak));
    assert!(!old_helper_hook_targets(weak).is_empty());
}

const ADVERSARIAL_LITERALS: &[&str] = &["secret_literal", "tail_secret", "root_secret"];
