use std::collections::BTreeSet;

use ferret_crypto::Prng;

pub struct IdentGenerator {
    rng: Prng,
    used: BTreeSet<String>,
}

impl IdentGenerator {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: Prng::new(seed ^ 0x1d35_7cab_f00d),
            used: BTreeSet::new(),
        }
    }

    pub fn ident(&mut self) -> String {
        loop {
            let len = 5 + self.rng.range(8);
            let mut name = String::with_capacity(len + 1);
            name.push(first_char(self.rng.range(53)));
            for _ in 1..len {
                name.push(next_char(self.rng.range(63)));
            }
            if !is_reserved_shape(&name) && self.used.insert(name.clone()) {
                return name;
            }
        }
    }
}

fn first_char(index: usize) -> char {
    match index {
        0..=25 => (b'a' + index as u8) as char,
        26..=51 => (b'A' + (index - 26) as u8) as char,
        _ => '_',
    }
}

fn next_char(index: usize) -> char {
    match index {
        0..=25 => (b'a' + index as u8) as char,
        26..=51 => (b'A' + (index - 26) as u8) as char,
        52..=61 => (b'0' + (index - 52) as u8) as char,
        _ => '_',
    }
}

fn is_reserved_shape(name: &str) -> bool {
    name.contains("_f_")
        || name.contains("OP_")
        || matches!(
            name,
            "and"
                | "break"
                | "do"
                | "else"
                | "elseif"
                | "end"
                | "false"
                | "for"
                | "function"
                | "goto"
                | "if"
                | "in"
                | "local"
                | "nil"
                | "not"
                | "or"
                | "repeat"
                | "return"
                | "then"
                | "true"
                | "until"
                | "while"
                | "_ENV"
                | "error"
                | "select"
                | "string"
                | "table"
                | "tonumber"
        )
}
