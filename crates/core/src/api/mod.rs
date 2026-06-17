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
    let emitted = ferret_vm::emit_lua(&compiled.chunk, options.seed);
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
