use std::collections::BTreeSet;

use ferret_crypto::Prng;

use crate::decoy_sections::{
    op_window, payload_constants, payload_section, payload_words, runner_section,
};
use crate::{IdentGenerator, OutputPlan};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecoyReport {
    pub blocks: usize,
    pub fake_opcodes: usize,
    pub fake_bytecode_words: usize,
}

pub fn decoy_block(plan: &OutputPlan, used_opcodes: &[u32]) -> (String, DecoyReport) {
    let mut idents = plan.idents(0xd3c0_0001);
    let mut numbers = plan.numbers(0xd3c0_0002);
    let mut rng = Prng::new(plan.seed(0xd3c0_0003));
    let ops = fake_ops(plan, used_opcodes, &mut idents, &mut rng);
    let mut out = String::new();
    out.push_str("do\n");
    for index in 0..plan.fake_payloads {
        let words = payload_words(plan, index);
        let constants = payload_constants(plan, index);
        let section_ops = op_window(&ops, index * 19, 18);
        payload_section(
            &mut out,
            &mut idents,
            &mut numbers,
            &mut rng,
            &section_ops,
            words,
            constants,
        );
    }
    for index in 0..plan.decoy_blocks {
        let section_ops = op_window(&ops, index * 13 + plan.fake_payloads, 12);
        runner_section(&mut out, &mut idents, &mut numbers, &mut rng, &section_ops);
    }
    out.push_str("end\n");
    (
        out,
        DecoyReport {
            blocks: plan.decoy_blocks + plan.fake_payloads,
            fake_opcodes: ops.len(),
            fake_bytecode_words: plan.fake_bytecode_words,
        },
    )
}

fn fake_ops(
    plan: &OutputPlan,
    used_opcodes: &[u32],
    idents: &mut IdentGenerator,
    rng: &mut Prng,
) -> Vec<(String, u32)> {
    let mut used = used_opcodes.iter().copied().collect::<BTreeSet<_>>();
    let mut ops = Vec::with_capacity(plan.fake_opcode_count);
    while ops.len() < plan.fake_opcode_count {
        let value = (rng.next_u32() & 0x7fff_ffff) | 1;
        if used.insert(value) {
            ops.push((idents.ident(), value));
        }
    }
    ops
}
