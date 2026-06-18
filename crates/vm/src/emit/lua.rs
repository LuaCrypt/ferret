use std::collections::BTreeMap;

use ferret_ir::{Chunk, Op};
use ferret_output::{
    decoy_block, rename_identifiers, rewrite_number_literals, Binding, IdentGenerator,
    NumberEncoder, OutputPlan, OutputStats, RuntimeTemplateVariant,
};
use ferret_util::stable_hash;

use crate::bytecode::layout::opcode_layout;
use crate::emit::constants::constants;
use crate::emit::lists::words;
use crate::emit::names::op_name;
use crate::emit::pack::encoded_words;
use crate::emit::runtime::{runtime, RuntimeInput};
use crate::emit::symbols::symbols;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmitReport {
    pub code: String,
    pub bytecode_words: usize,
    pub constant_count: usize,
    pub output_hash: u64,
    pub runtime_names_obfuscated: bool,
    pub static_decoys: usize,
    pub fake_opcode_count: usize,
    pub fake_bytecode_words: usize,
    pub runtime_template_variant: u8,
    pub output_hardening_level: u8,
    pub output_hardened: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmitOptions {
    pub profile: OutputProfile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputProfile {
    Lean,
    Hardened,
}

impl Default for EmitOptions {
    fn default() -> Self {
        Self {
            profile: OutputProfile::Hardened,
        }
    }
}

pub fn emit_lua(chunk: &Chunk, seed: u64) -> EmitReport {
    emit_lua_with_options(chunk, seed, EmitOptions::default())
}

pub fn emit_lua_with_options(chunk: &Chunk, seed: u64, options: EmitOptions) -> EmitReport {
    let layout = opcode_layout(seed);
    let syms = symbols(seed);
    let (enc_words, stream_seed) = encoded_words(chunk, &layout, seed, 0x70f0_1eaf);
    let plan = OutputPlan::new(
        seed,
        OutputStats {
            bytecode_words: enc_words.len(),
            constant_count: chunk.constants.len(),
            opcode_count: layout.len(),
        },
    );
    let hardened = options.profile == OutputProfile::Hardened;
    let runtime_variant = if hardened {
        plan.runtime_template_variant
    } else {
        RuntimeTemplateVariant::Compact
    };
    let mut numbers = plan.numbers(0x4e75_1eed);
    let mut word_text = String::new();
    words(&mut word_text, &enc_words, &mut numbers);
    let mut constant_text = String::new();
    constants(
        &mut constant_text,
        &chunk.constants,
        seed,
        &layout,
        &mut numbers,
    );
    let mut op_text = String::new();
    op_locals(&mut op_text, &layout, &mut numbers);
    let mut code = String::new();
    code.push_str("do\n");
    code.push_str(&runtime(RuntimeInput {
        seed: stream_seed,
        syms: &syms,
        op_text: &op_text,
        word_text: &word_text,
        constant_text: &constant_text,
        word_count: enc_words.len(),
        reuse_root_registers: !has_function_constants(&chunk.constants),
        variant: runtime_variant,
    }));
    code.push_str("\nend\n");
    let mut static_decoys = 0;
    let mut fake_opcode_count = 0;
    let mut fake_bytecode_words = 0;
    if hardened {
        let used_opcodes = layout.values().copied().collect::<Vec<_>>();
        let (decoys, report) = decoy_block(&plan, &used_opcodes);
        static_decoys = report.blocks;
        fake_opcode_count = report.fake_opcodes;
        fake_bytecode_words = report.fake_bytecode_words;
        code.push_str(&decoys);
    }
    code = harden_runtime_names(code, &layout, seed);
    let mut rewrite_numbers = plan.numbers(0x00bf_8ca1);
    code = rewrite_number_literals(&code, &mut rewrite_numbers);
    EmitReport {
        output_hash: stable_hash(code.as_bytes()),
        code,
        bytecode_words: enc_words.len(),
        constant_count: chunk.constants.len(),
        runtime_names_obfuscated: true,
        static_decoys,
        fake_opcode_count,
        fake_bytecode_words,
        runtime_template_variant: runtime_variant.id(),
        output_hardening_level: if hardened { plan.hardening_level } else { 0 },
        output_hardened: hardened,
    }
}

fn op_locals(out: &mut String, layout: &BTreeMap<Op, u32>, numbers: &mut NumberEncoder) {
    for (op, value) in layout {
        out.push_str("local ");
        out.push_str(op_name(*op));
        out.push('=');
        out.push_str(&numbers.u32(*value));
        out.push('\n');
    }
}

fn harden_runtime_names(code: String, layout: &BTreeMap<Op, u32>, seed: u64) -> String {
    let mut idents = IdentGenerator::new(seed ^ 0xa11a_5eed);
    let mut bindings = Vec::new();
    for op in layout.keys() {
        bindings.push(binding(op_name(*op), &mut idents));
    }
    for name in runtime_identifiers() {
        bindings.push(binding(name, &mut idents));
    }
    rename_identifiers(&code, &bindings)
}

fn binding(from: &str, idents: &mut IdentGenerator) -> Binding {
    Binding {
        from: from.to_string(),
        to: idents.ident(),
    }
}

fn runtime_identifiers() -> &'static [&'static str] {
    &[
        "_env",
        "_fc",
        "_entry",
        "_entry_fn",
        "_root_r",
        "_u",
        "_tc",
        "_ch",
        "_num",
        "_sel",
        "T",
        "seed",
        "s",
        "i",
        "O",
        "A",
        "B",
        "C",
        "j",
        "b",
        "o",
        "KC",
        "ci",
        "r",
        "t",
        "v",
        "k",
        "cache",
        "R",
        "U",
        "FW",
        "FC",
        "P",
        "CAP",
        "NU",
        "c",
        "FN",
        "W",
        "N",
        "a1",
        "a2",
        "a3",
        "WA",
        "WB",
        "WC",
        "pc",
        "op",
        "a",
        "ep",
        "mop",
        "ma",
        "mb",
        "mc",
        "mk",
        "dk",
        "f",
        "l",
        "n",
        "v1",
        "v2",
        "v3",
        "V",
    ]
}

fn has_function_constants(constants: &[ferret_ir::Const]) -> bool {
    constants
        .iter()
        .any(|constant| matches!(constant, ferret_ir::Const::Function { .. }))
}
