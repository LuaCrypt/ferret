use std::collections::BTreeMap;

use ferret_crypto::{encode_words, Prng};
use ferret_ir::{Chunk, Op};

pub(super) fn encoded_words(
    chunk: &Chunk,
    layout: &BTreeMap<Op, u32>,
    seed: u64,
    salt: u64,
) -> (Vec<u32>, u64) {
    let plain = pack_plain(chunk, layout);
    let stream_seed = stream_seed(seed, salt, &plain);
    let noisy = with_noise(plain, layout, stream_seed);
    (encode_words(&noisy, stream_seed), stream_seed)
}

fn pack_plain(chunk: &Chunk, layout: &BTreeMap<Op, u32>) -> Vec<u32> {
    let mut words = Vec::with_capacity(chunk.instructions.len() * 4);
    for instr in &chunk.instructions {
        words.push(layout[&instr.op]);
        words.push(u32::from(instr.a));
        words.push(u32::from(instr.b));
        words.push(u32::from(instr.c));
    }
    words
}

fn with_noise(mut words: Vec<u32>, layout: &BTreeMap<Op, u32>, stream_seed: u64) -> Vec<u32> {
    let used = layout.values().copied().collect::<Vec<_>>();
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
