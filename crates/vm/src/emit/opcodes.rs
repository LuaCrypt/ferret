use std::collections::{BTreeMap, BTreeSet};

use ferret_crypto::Prng;
use ferret_ir::Op;
use ferret_output::IdentGenerator;

#[derive(Debug, Clone)]
pub(super) struct OpcodePlan {
    primary: BTreeMap<Op, u32>,
    aliases: BTreeMap<Op, Vec<OpcodeAlias>>,
}

#[derive(Debug, Clone)]
pub(super) struct OpcodeAlias {
    pub(super) name: String,
    pub(super) value: u32,
}

impl OpcodePlan {
    pub(super) fn new(primary: BTreeMap<Op, u32>, seed: u64, enabled: bool) -> Self {
        let aliases = if enabled {
            aliases(&primary, seed)
        } else {
            BTreeMap::new()
        };
        Self { primary, aliases }
    }

    pub(super) fn primary(&self) -> &BTreeMap<Op, u32> {
        &self.primary
    }

    pub(super) fn code(&self, op: Op) -> u32 {
        self.primary[&op]
    }

    pub(super) fn aliases_for(&self, op: Op) -> &[OpcodeAlias] {
        self.aliases.get(&op).map(Vec::as_slice).unwrap_or(&[])
    }

    pub(super) fn code_for(&self, op: Op, rng: &mut Prng, allow_alias: bool) -> u32 {
        let aliases = self.aliases_for(op);
        if allow_alias && !aliases.is_empty() {
            aliases[rng.range(aliases.len())].value
        } else {
            self.code(op)
        }
    }

    pub(super) fn alias_count(&self) -> usize {
        self.aliases.values().map(Vec::len).sum()
    }

    pub(super) fn used_values(&self) -> Vec<u32> {
        let mut values = self.primary.values().copied().collect::<Vec<_>>();
        for aliases in self.aliases.values() {
            values.extend(aliases.iter().map(|alias| alias.value));
        }
        values
    }
}

fn aliases(primary: &BTreeMap<Op, u32>, seed: u64) -> BTreeMap<Op, Vec<OpcodeAlias>> {
    let mut used = primary.values().copied().collect::<BTreeSet<_>>();
    let mut names = IdentGenerator::new(seed ^ 0xa11a_501d);
    let mut rng = Prng::new(seed ^ 0xa11a_c0de);
    let mut out = BTreeMap::new();
    for op in alias_ops() {
        let value = unique_code(&mut rng, &mut used);
        out.insert(
            op,
            vec![OpcodeAlias {
                name: names.ident(),
                value,
            }],
        );
    }
    out
}

fn alias_ops() -> [Op; 5] {
    [Op::LoadK, Op::Move, Op::GetGlobal, Op::CallGlobal, Op::Halt]
}

fn unique_code(rng: &mut Prng, used: &mut BTreeSet<u32>) -> u32 {
    loop {
        let value = (rng.next_u32() & 0x7fff_ffff) | 1;
        if used.insert(value) {
            return value;
        }
    }
}
