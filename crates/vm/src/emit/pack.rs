use ferret_crypto::{encode_words, Prng};
use ferret_ir::{Chunk, Op};

use crate::emit::opcodes::OpcodePlan;

pub(super) fn encoded_words(
    chunk: &Chunk,
    opcodes: &OpcodePlan,
    seed: u64,
    salt: u64,
) -> (Vec<u32>, u64) {
    let plain = pack_plain(chunk, opcodes, seed ^ salt);
    let stream_seed = stream_seed(seed, salt, &plain);
    let noisy = with_noise(plain, opcodes, stream_seed);
    (encode_words(&noisy, stream_seed), stream_seed)
}

fn pack_plain(chunk: &Chunk, opcodes: &OpcodePlan, seed: u64) -> Vec<u32> {
    let mut words = Vec::with_capacity(chunk.instructions.len() * 4);
    let mut alias_rng = Prng::new(seed ^ 0xa11a_1ced);
    let mut superblock_body = 0usize;
    for instr in &chunk.instructions {
        let allow_alias = superblock_body == 0;
        words.push(opcodes.code_for(instr.op, &mut alias_rng, allow_alias));
        words.push(u32::from(instr.a));
        words.push(u32::from(instr.b));
        words.push(u32::from(instr.c));
        if superblock_body > 0 {
            superblock_body -= 1;
        } else if instr.op == Op::SuperBlock {
            superblock_body = usize::from(instr.a);
        }
    }
    words
}

fn with_noise(mut words: Vec<u32>, opcodes: &OpcodePlan, stream_seed: u64) -> Vec<u32> {
    let used = opcodes.used_values();
    let mut rng = Prng::new(stream_seed ^ 0x6a09_dead);
    let count = 1 + rng.range(3);
    for _ in 0..count {
        words.push(dead_opcode(&mut rng, &used));
        words.push(rng.next_u32() & 0xffff);
        words.push(rng.next_u32() & 0xffff);
        words.push(rng.next_u32() & 0xffff);
    }
    words
}

fn dead_opcode(rng: &mut Prng, used: &[u32]) -> u32 {
    loop {
        let candidate = (rng.next_u32() & 0x7fff_ffff) | 1;
        if !used.contains(&candidate) {
            return candidate;
        }
    }
}

fn stream_seed(seed: u64, salt: u64, words: &[u32]) -> u64 {
    let mut hash = seed ^ salt ^ 0xcbf2_9ce4_8422_2325;
    for word in words {
        hash ^= u64::from(*word);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use ferret_ir::{Chunk, Instr, Op};

    use super::pack_plain;
    use crate::emit::opcodes::OpcodePlan;

    #[test]
    fn aliases_are_used_outside_superblock_bodies() {
        let primary = primary_layout();
        let opcodes = OpcodePlan::new(primary, 77, true);
        let chunk = Chunk {
            constants: Vec::new(),
            instructions: vec![
                Instr::new(Op::LoadK, 1, 0, 0),
                Instr::new(Op::SuperBlock, 2, 0, 0),
                Instr::new(Op::LoadK, 2, 0, 0),
                Instr::new(Op::Move, 3, 2, 0),
                Instr::new(Op::Halt, 0, 0, 0),
            ],
            registers: 4,
            params: 0,
        };
        let words = pack_plain(&chunk, &opcodes, 19);
        assert!(opcodes
            .aliases_for(Op::LoadK)
            .iter()
            .any(|alias| alias.value == words[0]));
        assert_eq!(words[8], opcodes.code(Op::LoadK));
        assert_eq!(words[12], opcodes.code(Op::Move));
        assert!(opcodes
            .aliases_for(Op::Halt)
            .iter()
            .any(|alias| alias.value == words[16]));
    }

    fn primary_layout() -> BTreeMap<Op, u32> {
        [Op::LoadK, Op::Move, Op::SuperBlock, Op::Halt]
            .into_iter()
            .enumerate()
            .map(|(index, op)| (op, (index as u32 + 1) * 11))
            .collect()
    }
}
