use std::fs;
use std::path::Path;

use ferret_core::{obfuscate, ObfuscationOptions};

mod support;
use support::{assert_hardened_output_shape, lua_available, run_lua, run_lua_source};

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
        assert!(result.metadata.runtime_names_obfuscated);
        assert!(result.metadata.output_hardened);
        assert!(result.metadata.static_decoys > 0);
        assert!(result.metadata.fake_opcode_count > 0);
        assert!(result.metadata.fake_bytecode_words > result.metadata.bytecode_word_count);
        assert!(result.metadata.semantic_alias_count > 0);
        assert!(result.metadata.handler_polymorphism_level > 0);
        assert!(result.metadata.output_hardening_level > 0);
        assert!(result.metadata.dump_resistance_level > 0);
        assert_eq!(result.metadata.adversarial_suite_version, 1);
        assert_eq!(result.metadata.semantic_mismatch_exclusions, 0);
        assert!(result.metadata.env_semantics);
        assert!(result.metadata.open_tail_multireturn);
        assert!(result.metadata.table_tail_multireturn);
        assert!(result.metadata.runtime_integrity_checks);
        assert!(result.metadata.delayed_string_constants);
        assert_ne!(result.metadata.bytecode_integrity_tag, 0);
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
    assert!(first.metadata.output_hardened);
    assert!(first.metadata.runtime_names_obfuscated);
    assert!(first.metadata.static_decoys > 0);
    assert!(first.metadata.fake_opcode_count > 70);
    assert!(first.metadata.fake_bytecode_words >= first.metadata.bytecode_word_count * 3);
    assert!(first.metadata.semantic_alias_count > 0);
    assert!(first.metadata.handler_polymorphism_level > 0);
    assert!(first.metadata.dump_resistance_level > 0);
    assert_eq!(first.metadata.adversarial_suite_version, 1);
    assert_eq!(first.metadata.semantic_mismatch_exclusions, 0);
    assert!(first.metadata.env_semantics);
    assert!(first.metadata.open_tail_multireturn);
    assert!(first.metadata.table_tail_multireturn);
    assert!(first.metadata.runtime_integrity_checks);
    assert!(first.metadata.delayed_string_constants);
    assert_ne!(first.metadata.bytecode_integrity_tag, 0);
    assert_hardened_output_shape(&first.code, &["basic"]);

    let third = obfuscate(
        &source,
        ObfuscationOptions {
            seed: 6,
            ..ObfuscationOptions::default()
        },
    )
    .unwrap();
    assert_ne!(first.code, third.code);
    assert_ne!(
        first.metadata.runtime_template_variant,
        third.metadata.runtime_template_variant
    );
    assert_ne!(
        first.metadata.bytecode_layout_variant,
        third.metadata.bytecode_layout_variant
    );
    assert_ne!(
        first.metadata.constant_layout_variant,
        third.metadata.constant_layout_variant
    );
    assert_ne!(
        first.metadata.bytecode_integrity_tag,
        third.metadata.bytecode_integrity_tag
    );
}

#[test]
fn strong_output_hides_runtime_shape_and_adds_static_decoys() {
    let source = "local total = 'hidden'\nprint(total)\n";
    let result = obfuscate(
        source,
        ObfuscationOptions {
            seed: 123,
            ..ObfuscationOptions::default()
        },
    )
    .unwrap();
    assert_hardened_output_shape(&result.code, &["hidden", "total", "print"]);
    let boundary = result.code.rfind("\nend\ndo\n").unwrap();
    assert!(boundary + "\nend\ndo\n".len() < result.code.len());
    assert!(result.metadata.fake_opcode_count > 70);
    assert!(result.metadata.fake_bytecode_words >= result.metadata.bytecode_word_count * 3);
    assert!(result.metadata.semantic_alias_count > 0);
    assert!(result.metadata.dump_resistance_level > 0);
    assert_eq!(result.metadata.adversarial_suite_version, 1);
    assert!(result.metadata.runtime_integrity_checks);
    assert!(result.metadata.delayed_string_constants);

    if lua_available() {
        let temp = tempfile::NamedTempFile::new().unwrap();
        fs::write(temp.path(), result.code).unwrap();
        let output = run_lua(temp.path());
        assert!(output.status.success());
        assert_eq!(output.stdout, b"hidden\n");
    }
}

#[test]
fn lexical_env_scope_restores_after_block() {
    if !lua_available() {
        return;
    }
    let source = r#"
local outer_print = print
do
    local _ENV = { print = outer_print, value = 91 }
    print("env_scope", value)
end
print("env_scope_after", type(value))
"#;
    let result = obfuscate(
        source,
        ObfuscationOptions {
            seed: 321,
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

#[test]
fn supports_single_argument_call_syntax() {
    if !lua_available() {
        return;
    }
    let source = r#"
print "string_call"
local t = setmetatable({}, { __call = function(_, value) print("table_call", value[1]) end })
t { "ok" }
"#;
    let result = obfuscate(source, ObfuscationOptions::default()).unwrap();
    let temp = tempfile::NamedTempFile::new().unwrap();
    fs::write(temp.path(), result.code).unwrap();
    let original = run_lua_source(source);
    let obfuscated = run_lua(temp.path());
    assert_eq!(original.status.code(), obfuscated.status.code());
    assert_eq!(original.stdout, obfuscated.stdout);
}

#[test]
fn rejects_dynamic_loader() {
    let err = obfuscate("load('print(1)')()", ObfuscationOptions::default()).unwrap_err();
    assert!(err.to_string().contains("load"));
}

#[test]
fn rejects_debug_lib_references() {
    for source in [
        "local x = debug\nprint(x.getinfo)",
        "local x = _ENV.debug\nprint(x.getinfo)",
        "local x = require('debug')\nprint(x.getinfo)",
    ] {
        let err = obfuscate(source, ObfuscationOptions::default()).unwrap_err();
        assert!(err.to_string().contains("debug"), "{source}");
    }
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
        script_path("env_globals/001_env_globals.lua"),
        script_path("env_globals/002_env_globals.lua"),
        script_path("env_globals/003_env_globals.lua"),
        script_path("env_globals/004_env_globals.lua"),
        script_path("env_globals/005_env_globals.lua"),
        script_path("env_globals/006_env_globals.lua"),
        script_path("env_globals/007_env_globals.lua"),
        script_path("env_globals/008_env_globals.lua"),
        script_path("functions/002_functions.lua"),
        script_path("functions/003_functions.lua"),
        script_path("generic_for/001_generic_for.lua"),
        script_path("generic_for/002_generic_for.lua"),
        script_path("goto_labels/001_goto_labels.lua"),
        script_path("regressions/001_regressions.lua"),
        script_path("varargs_multireturn/001_varargs_multireturn.lua"),
        script_path("varargs_multireturn/009_varargs_multireturn.lua"),
        lua54_path("001_multireturn_assignment.lua"),
        lua54_path("002_return_open_tail.lua"),
        lua54_path("003_table_final_multireturn.lua"),
        lua54_path("004_lexical_env_nested.lua"),
        lua54_path("005_goto_forward_backward.lua"),
        lua54_path("009_pcall_multireturn.lua"),
        lua54_path("010_vararg_return_tail.lua"),
    ]
}

fn lua54_path(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("tests/lua54_conformance")
        .join(name)
}
