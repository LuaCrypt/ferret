use crate::{IdentGenerator, NumberEncoder};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutputStats {
    pub bytecode_words: usize,
    pub constant_count: usize,
    pub opcode_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutputPlan {
    seed: u64,
    pub decoy_blocks: usize,
    pub fake_payloads: usize,
    pub fake_opcode_count: usize,
    pub fake_bytecode_words: usize,
    pub fake_constant_count: usize,
    pub runtime_template_variant: RuntimeTemplateVariant,
    pub hardening_level: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeTemplateVariant {
    Compact = 0,
    SwappedFetch = 1,
    SwappedCompare = 2,
    Mixed = 3,
}

impl OutputPlan {
    pub fn new(seed: u64, stats: OutputStats) -> Self {
        let spread = (seed.rotate_left(17) ^ seed.rotate_right(11)) as usize;
        let real_words = stats.bytecode_words.max(32);
        let fake_multiplier = 3 + (spread % 2);
        let fake_words = round_quad((real_words * fake_multiplier).max(128));
        let fake_constants = (stats.constant_count.max(4) * 3 + 16 + (spread % 9)).max(28);
        let fake_ops = (stats.opcode_count.max(16) + 24 + (spread % 7)).max(80);
        Self {
            seed,
            decoy_blocks: 4 + (spread % 2),
            fake_payloads: 2 + (spread % 2),
            fake_opcode_count: fake_ops,
            fake_bytecode_words: fake_words,
            fake_constant_count: fake_constants,
            runtime_template_variant: RuntimeTemplateVariant::from_seed(seed),
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
