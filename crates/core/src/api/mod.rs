use ferret_util::{stable_hash, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Preset {
    Balanced,
    #[default]
    Strong,
}

impl std::str::FromStr for Preset {
    type Err = ferret_util::FerretError;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "balanced" => Ok(Self::Balanced),
            "strong" => Ok(Self::Strong),
            _ => Err(ferret_util::FerretError::Compile(format!(
                "unknown preset '{value}'"
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObfuscationOptions {
    pub seed: u64,
    pub preset: Preset,
}

impl Default for ObfuscationOptions {
    fn default() -> Self {
        Self {
            seed: 0xF3EE_2026,
            preset: Preset::Strong,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Metadata {
    pub lua_version: String,
    pub preset: Preset,
    pub seed: u64,
    pub input_hash: u64,
    pub output_hash: u64,
    pub vm_only: bool,
    pub source_reconstruction: bool,
    pub instruction_count: usize,
    pub bytecode_word_count: usize,
    pub constant_count: usize,
    pub encrypted_bytecode: bool,
    pub encrypted_constants: bool,
    pub custom_opcodes: bool,
    pub runtime_names_obfuscated: bool,
    pub static_decoys: usize,
    pub fake_opcode_count: usize,
    pub fake_bytecode_words: usize,
    pub bytecode_integrity_tag: u32,
    pub runtime_template_variant: u8,
    pub bytecode_layout_variant: u8,
    pub constant_layout_variant: u8,
    pub semantic_alias_count: usize,
    pub handler_polymorphism_level: u8,
    pub output_hardening_level: u8,
    pub runtime_integrity_checks: bool,
    pub delayed_string_constants: bool,
    pub output_hardened: bool,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObfuscationResult {
    pub code: String,
    pub metadata: Metadata,
}

pub fn obfuscate(source: &str, options: ObfuscationOptions) -> Result<ObfuscationResult> {
    reject_hostile(source)?;
    let program = ferret_parse::parse(source)?;
    let compiled = ferret_vm::compile(&program)?;
    let profile = match options.preset {
        Preset::Balanced => ferret_vm::OutputProfile::Lean,
        Preset::Strong => ferret_vm::OutputProfile::Hardened,
    };
    let emitted = ferret_vm::emit_lua_with_options(
        &compiled.chunk,
        options.seed,
        ferret_vm::EmitOptions { profile },
    );
    let metadata = Metadata {
        lua_version: "lua54".to_string(),
        preset: options.preset,
        seed: options.seed,
        input_hash: stable_hash(source.as_bytes()),
        output_hash: emitted.output_hash,
        vm_only: true,
        source_reconstruction: false,
        instruction_count: compiled.chunk.instructions.len(),
        bytecode_word_count: emitted.bytecode_words,
        constant_count: emitted.constant_count,
        encrypted_bytecode: true,
        encrypted_constants: true,
        custom_opcodes: true,
        runtime_names_obfuscated: emitted.runtime_names_obfuscated,
        static_decoys: emitted.static_decoys,
        fake_opcode_count: emitted.fake_opcode_count,
        fake_bytecode_words: emitted.fake_bytecode_words,
        bytecode_integrity_tag: emitted.bytecode_integrity_tag,
        runtime_template_variant: emitted.runtime_template_variant,
        bytecode_layout_variant: emitted.bytecode_layout_variant,
        constant_layout_variant: emitted.constant_layout_variant,
        semantic_alias_count: emitted.semantic_alias_count,
        handler_polymorphism_level: emitted.handler_polymorphism_level,
        output_hardening_level: emitted.output_hardening_level,
        runtime_integrity_checks: emitted.runtime_integrity_checks,
        delayed_string_constants: emitted.delayed_string_constants,
        output_hardened: emitted.output_hardened,
        limitations: vec![
            "open-source runtime can be instrumented".to_string(),
            "unsupported Lua constructs fail instead of falling back".to_string(),
            "closures, goto, repeat-until, and to-be-closed variables are not supported yet"
                .to_string(),
        ],
    };
    Ok(ObfuscationResult {
        code: emitted.code,
        metadata,
    })
}

fn reject_hostile(source: &str) -> Result<()> {
    for word in ["load", "loadfile", "dofile", "debug", "coroutine.yield"] {
        if source.contains(word) {
            return Err(ferret_util::FerretError::Unsupported(format!(
                "'{word}' is rejected by the VM-only profile"
            )));
        }
    }
    Ok(())
}
