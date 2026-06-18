use std::collections::BTreeSet;

use ferret_crypto::Prng;

use crate::decoy_sections::{
    op_window, payload_constants, payload_section, payload_words, runner_section,
};
use crate::{HardeningProfile, IdentGenerator};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecoyReport {
    pub blocks: usize,
    pub fake_opcodes: usize,
    pub fake_bytecode_words: usize,
}

pub fn decoy_block(profile: &HardeningProfile, used_opcodes: &[u32]) -> (String, DecoyReport) {
    let mut idents = profile.idents(0xd3c0_0001);
    let mut numbers = profile.numbers(0xd3c0_0002);
    let mut rng = Prng::new(profile.seed(0xd3c0_0003));
    let ops = fake_ops(profile, used_opcodes, &mut idents, &mut rng);
    let mut out = String::new();
    out.push_str("do\n");
    for index in 0..profile.fake_payloads {
        let words = payload_words(profile, index);
        let constants = payload_constants(profile, index);
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
    for index in 0..profile.decoy_blocks {
        let section_ops = op_window(&ops, index * 13 + profile.fake_payloads, 12);
        runner_section(&mut out, &mut idents, &mut numbers, &mut rng, &section_ops);
    }
    out.push_str("end\n");
    (
        out,
        DecoyReport {
            blocks: profile.decoy_blocks + profile.fake_payloads,
            fake_opcodes: ops.len(),
            fake_bytecode_words: profile.fake_bytecode_words,
        },
    )
}

fn fake_ops(
    profile: &HardeningProfile,
    used_opcodes: &[u32],
    idents: &mut IdentGenerator,
    rng: &mut Prng,
) -> Vec<(String, u32)> {
    let mut used = used_opcodes.iter().copied().collect::<BTreeSet<_>>();
    let mut ops = Vec::with_capacity(profile.fake_opcode_count);
    while ops.len() < profile.fake_opcode_count {
        let value = (rng.next_u32() & 0x7fff_ffff) | 1;
        if used.insert(value) {
            ops.push((idents.ident(), value));
        }
    }
    ops
}
