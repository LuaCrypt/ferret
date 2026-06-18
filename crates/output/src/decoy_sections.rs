use ferret_crypto::Prng;

use crate::{HardeningProfile, IdentGenerator, NumberEncoder};

pub(super) fn payload_section(
    out: &mut String,
    idents: &mut IdentGenerator,
    numbers: &mut NumberEncoder,
    rng: &mut Prng,
    ops: &[(String, u32)],
    word_count: usize,
    constant_count: usize,
) {
    out.push_str("do\n");
    opcode_locals(out, ops, numbers);
    let words = idents.ident();
    let constants = idents.ident();
    let cache = idents.ident();
    let chunk = idents.ident();
    let mapper = idents.ident();
    let loader = idents.ident();
    let input = idents.ident();
    let key = idents.ident();
    let index = idents.ident();
    let state = idents.ident();
    out.push_str("local ");
    out.push_str(&words);
    out.push('=');
    fake_words(out, numbers, rng, ops, word_count);
    out.push_str("\nlocal ");
    out.push_str(&constants);
    out.push('=');
    fake_constants(out, numbers, rng, constant_count);
    out.push_str("\nlocal ");
    out.push_str(&cache);
    out.push_str("={}\nlocal ");
    out.push_str(&chunk);
    out.push_str("={");
    out.push_str(&words);
    out.push(',');
    out.push_str(&constants);
    out.push(',');
    out.push_str(&cache);
    out.push_str("}\nlocal function ");
    out.push_str(&mapper);
    out.push('(');
    out.push_str(&input);
    out.push(',');
    out.push_str(&key);
    out.push_str(") local ");
    out.push_str(&state);
    out.push_str("={} for ");
    out.push_str(&index);
    out.push_str("=1,#");
    out.push_str(&input);
    out.push_str(" do ");
    out.push_str(&state);
    out.push('[');
    out.push_str(&index);
    out.push_str("]=(");
    out.push_str(&input);
    out.push('[');
    out.push_str(&index);
    out.push_str("]~(");
    out.push_str(&key);
    out.push('+');
    out.push_str(&index);
    out.push('*');
    out.push_str(&numbers.u32(97 + (rng.next_u32() & 255)));
    out.push_str("))&");
    out.push_str(&numbers.u32(0x7fff_ffff));
    out.push_str(" end return ");
    out.push_str(&state);
    out.push_str(" end\nlocal ");
    out.push_str(&loader);
    out.push('=');
    out.push_str(&mapper);
    out.push('(');
    out.push_str(&words);
    out.push(',');
    out.push_str(&numbers.u32(rng.next_u32()));
    out.push_str(")\nend\n");
}

pub(super) fn runner_section(
    out: &mut String,
    idents: &mut IdentGenerator,
    numbers: &mut NumberEncoder,
    rng: &mut Prng,
    ops: &[(String, u32)],
) {
    out.push_str("do\n");
    opcode_locals(out, ops, numbers);
    let run = idents.ident();
    let tape = idents.ident();
    let slot = idents.ident();
    let op = idents.ident();
    let acc = idents.ident();
    out.push_str("local function ");
    out.push_str(&run);
    out.push('(');
    out.push_str(&tape);
    out.push_str(") local ");
    out.push_str(&slot);
    out.push_str("=1 local ");
    out.push_str(&acc);
    out.push_str("=0 while ");
    out.push_str(&slot);
    out.push_str("<=#");
    out.push_str(&tape);
    out.push_str(" do local ");
    out.push_str(&op);
    out.push('=');
    out.push_str(&tape);
    out.push('[');
    out.push_str(&slot);
    out.push_str("] ");
    fake_cases(out, numbers, rng, ops, &op, &acc);
    out.push_str(&slot);
    out.push('=');
    out.push_str(&slot);
    out.push_str("+4 end return ");
    out.push_str(&acc);
    out.push_str(" end\nend\n");
}

pub(super) fn payload_words(profile: &HardeningProfile, index: usize) -> usize {
    let base = profile.fake_bytecode_words / profile.fake_payloads;
    let extra = usize::from(index < profile.fake_bytecode_words % profile.fake_payloads);
    round_quad(base + extra)
}

pub(super) fn payload_constants(profile: &HardeningProfile, index: usize) -> usize {
    let base = profile.fake_constant_count / profile.fake_payloads;
    let extra = usize::from(index < profile.fake_constant_count % profile.fake_payloads);
    (base + extra).max(1)
}

pub(super) fn op_window(ops: &[(String, u32)], start: usize, count: usize) -> Vec<(String, u32)> {
    (0..count)
        .map(|offset| ops[(start + offset) % ops.len()].clone())
        .collect()
}

fn opcode_locals(out: &mut String, ops: &[(String, u32)], numbers: &mut NumberEncoder) {
    for group in ops.chunks(6) {
        out.push_str("local ");
        for (index, (name, _)) in group.iter().enumerate() {
            if index > 0 {
                out.push(',');
            }
            out.push_str(name);
        }
        out.push('=');
        for (index, (_, value)) in group.iter().enumerate() {
            if index > 0 {
                out.push(',');
            }
            out.push_str(&numbers.u32(*value));
        }
        out.push('\n');
    }
}

fn fake_words(
    out: &mut String,
    numbers: &mut NumberEncoder,
    rng: &mut Prng,
    ops: &[(String, u32)],
    word_count: usize,
) {
    out.push('{');
    for index in 0..word_count {
        let value = if index % 4 == 0 {
            ops[rng.range(ops.len())].1
        } else {
            rng.next_u32() & 0xffff
        };
        out.push_str(&numbers.u32(value));
        out.push(',');
    }
    out.push('}');
}

fn fake_constants(
    out: &mut String,
    numbers: &mut NumberEncoder,
    rng: &mut Prng,
    constant_count: usize,
) {
    out.push_str("{{");
    for _ in 0..constant_count {
        let tag = 1 + rng.range(3);
        out.push('{');
        out.push_str(&numbers.u8(tag as u8));
        out.push(',');
        out.push_str(&numbers.u32(rng.next_u32()));
        out.push(',');
        let byte_count = 4 + rng.range(10);
        byte_table(out, numbers, rng, byte_count);
        out.push_str("},");
    }
    out.push_str("},{");
    for index in 1..=constant_count {
        out.push_str(&numbers.usize(index));
        out.push(',');
    }
    out.push_str("},{}}");
}

fn byte_table(out: &mut String, numbers: &mut NumberEncoder, rng: &mut Prng, len: usize) {
    out.push('{');
    for _ in 0..len {
        out.push_str(&numbers.u8((rng.next_u32() & 255) as u8));
        out.push(',');
    }
    out.push('}');
}

fn fake_cases(
    out: &mut String,
    numbers: &mut NumberEncoder,
    rng: &mut Prng,
    ops: &[(String, u32)],
    op: &str,
    acc: &str,
) {
    for branch in 0..6 {
        out.push_str(if branch == 0 { "if " } else { "elseif " });
        let name = &ops[rng.range(ops.len())].0;
        if branch % 2 == 0 {
            out.push_str(op);
            out.push_str("==");
            out.push_str(name);
        } else {
            out.push_str(name);
            out.push_str("==");
            out.push_str(op);
        }
        out.push_str(" then ");
        out.push_str(acc);
        out.push('=');
        out.push_str(acc);
        out.push(if branch % 3 == 0 { '~' } else { '+' });
        out.push_str(&numbers.u32(rng.next_u32() & 0xffff));
        out.push(' ');
    }
    out.push_str("else ");
    out.push_str(acc);
    out.push('=');
    out.push_str(acc);
    out.push_str(" end ");
}

fn round_quad(value: usize) -> usize {
    value + ((4 - (value % 4)) % 4)
}
