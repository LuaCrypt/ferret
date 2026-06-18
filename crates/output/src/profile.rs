use crate::{IdentGenerator, NumberEncoder};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProfileStats {
    pub bytecode_words: usize,
    pub constant_count: usize,
    pub opcode_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HardeningProfile {
    seed: u64,
    pub bytecode_layout: BytecodeLayout,
    pub constant_layout: ConstantLayout,
    pub decoy_blocks: usize,
    pub fake_payloads: usize,
    pub fake_opcode_count: usize,
    pub fake_bytecode_words: usize,
    pub fake_constant_count: usize,
    pub runtime_template_variant: RuntimeTemplateVariant,
    pub handler_polymorphism_level: u8,
    pub hardening_level: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BytecodeLayout {
    pub opcode: usize,
    pub a: usize,
    pub b: usize,
    pub c: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConstantLayout {
    pub rows: usize,
    pub map: usize,
    pub cache: usize,
    pub state: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeTemplateVariant {
    Compact = 0,
    SwappedFetch = 1,
    SwappedCompare = 2,
    Mixed = 3,
}

impl HardeningProfile {
    pub fn new(seed: u64, stats: ProfileStats) -> Self {
        let spread = (seed.rotate_left(17) ^ seed.rotate_right(11)) as usize;
        let real_words = stats.bytecode_words.max(32);
        let fake_multiplier = 3 + (spread % 2);
        let fake_words = round_quad((real_words * fake_multiplier).max(96));
        let fake_constants = (stats.constant_count.max(4) * 2 + 12 + (spread % 7)).max(20);
        let fake_ops = (stats.opcode_count.max(16) + 16 + (spread % 5)).max(72);
        Self {
            seed,
            bytecode_layout: BytecodeLayout::from_seed(seed ^ 0xb17e_c01a),
            constant_layout: ConstantLayout::from_seed(seed ^ 0xc057_5a10),
            decoy_blocks: 3 + (spread % 2),
            fake_payloads: 2 + (spread % 2),
            fake_opcode_count: fake_ops,
            fake_bytecode_words: fake_words,
            fake_constant_count: fake_constants,
            runtime_template_variant: RuntimeTemplateVariant::from_seed(seed),
            handler_polymorphism_level: 2 + ((spread as u8) & 1),
            hardening_level: 2,
        }
    }

    pub fn idents(self, salt: u64) -> IdentGenerator {
        IdentGenerator::new(self.seed(salt))
    }

    pub fn numbers(self, salt: u64) -> NumberEncoder {
        NumberEncoder::new(self.seed(salt))
    }

    pub fn seed(self, salt: u64) -> u64 {
        self.seed ^ salt.wrapping_mul(0x9e37_79b9_7f4a_7c15)
    }
}

impl BytecodeLayout {
    fn from_seed(seed: u64) -> Self {
        let slots = shuffled_slots(seed);
        Self {
            opcode: slots[0],
            a: slots[1],
            b: slots[2],
            c: slots[3],
        }
    }

    pub fn id(self) -> u8 {
        packed_id([self.opcode, self.a, self.b, self.c])
    }
}

impl ConstantLayout {
    fn from_seed(seed: u64) -> Self {
        let slots = shuffled_slots(seed);
        Self {
            rows: slots[0],
            map: slots[1],
            cache: slots[2],
            state: slots[3],
        }
    }

    pub fn id(self) -> u8 {
        packed_id([self.rows, self.map, self.cache, self.state])
    }
}

impl RuntimeTemplateVariant {
    const COUNT: u8 = 4;

    fn from_seed(seed: u64) -> Self {
        match (seed as u8).wrapping_add(2) % Self::COUNT {
            0 => Self::Compact,
            1 => Self::SwappedFetch,
            2 => Self::SwappedCompare,
            _ => Self::Mixed,
        }
    }

    pub fn id(self) -> u8 {
        self as u8
    }
}

fn round_quad(value: usize) -> usize {
    value + ((4 - (value % 4)) % 4)
}

fn shuffled_slots(seed: u64) -> [usize; 4] {
    let mut slots = [1, 2, 3, 4];
    let mut value = seed;
    for index in (1..slots.len()).rev() {
        value ^= value >> 12;
        value ^= value << 25;
        value ^= value >> 27;
        let swap = ((value.wrapping_mul(0x2545_f491_4f6c_dd1d) >> 32) as usize) % (index + 1);
        slots.swap(index, swap);
    }
    slots
}

fn packed_id(slots: [usize; 4]) -> u8 {
    slots
        .into_iter()
        .fold(0u8, |id, slot| (id << 2) | ((slot as u8 - 1) & 3))
}
