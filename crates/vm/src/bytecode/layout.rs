use std::collections::BTreeMap;

use ferret_crypto::Prng;
use ferret_ir::{Op, OPCODE_DEFS};

pub fn opcode_layout(seed: u64) -> BTreeMap<Op, u32> {
    let mut rng = Prng::new(seed ^ 0x0f0f_b17e);
    let mut used = Vec::new();
    let mut map = BTreeMap::new();
    for (op, _) in OPCODE_DEFS {
        let code = unique_code(&mut rng, &used);
        used.push(code);
        map.insert(*op, code);
    }
    map
}

fn unique_code(rng: &mut Prng, used: &[u32]) -> u32 {
    loop {
        let value = (rng.next_u32() & 0x7fff_ffff) | 1;
        if !used.contains(&value) {
            return value;
        }
    }
}
