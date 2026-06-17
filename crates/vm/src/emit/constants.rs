use std::collections::BTreeMap;

use ferret_crypto::{encode_bytes, Prng};
use ferret_ir::{Capture, Chunk, Const, Op};

use crate::emit::lists::{bytes, words};
use crate::emit::pack::encoded_words;

pub(super) fn constants(
    out: &mut String,
    constants: &[Const],
    seed: u64,
    layout: &BTreeMap<Op, u32>,
) {
    let order = shuffled_order(constants.len(), seed);
    let mut map = vec![0usize; constants.len()];
    for (slot, index) in order.iter().copied().enumerate() {
        map[index] = slot + 1;
    }
    out.push_str("{{");
    for index in order {
        constant(out, &constants[index], item_seed(seed, index), layout);
    }
    out.push_str("},{");
    for slot in map {
        out.push_str(&slot.to_string());
        out.push(',');
    }
    out.push_str("},{}}");
}

fn constant(out: &mut String, constant: &Const, seed: u64, layout: &BTreeMap<Op, u32>) {
    match constant {
        Const::Nil => out.push_str("{0},"),
        Const::Bool(value) => {
            out.push_str(if *value { "{1,1}," } else { "{1,0}," });
        }
        Const::Number(value) => protected(out, 2, &number_text(*value), seed),
        Const::String(value) => protected(out, 3, value, seed),
        Const::Function { chunk, captures } => function_const(out, chunk, captures, seed, layout),
    }
}

fn function_const(
    out: &mut String,
    chunk: &Chunk,
    captures: &[Capture],
    seed: u64,
    layout: &BTreeMap<Op, u32>,
) {
    let (encoded, stream_seed) = encoded_words(chunk, layout, seed, 0xf17e_f00d);
    out.push_str("{4,");
    out.push_str(&(stream_seed as u32).to_string());
    out.push(',');
    words(out, &encoded);
    out.push(',');
    constants(out, &chunk.constants, stream_seed, layout);
    out.push(',');
    out.push_str(&chunk.params.to_string());
    out.push(',');
    capture_list(out, captures);
    out.push_str("},");
}

fn capture_list(out: &mut String, captures: &[Capture]) {
    out.push('{');
    for capture in captures {
        match capture {
            Capture::Local(reg) => capture_item(out, 0, *reg),
            Capture::Upvalue(index) => capture_item(out, 1, *index),
        }
    }
    out.push('}');
}

fn capture_item(out: &mut String, tag: u8, value: u16) {
    out.push('{');
    out.push_str(&tag.to_string());
    out.push(',');
    out.push_str(&value.to_string());
    out.push_str("},");
}

fn protected(out: &mut String, tag: u8, value: &str, seed: u64) {
    out.push('{');
    out.push_str(&tag.to_string());
    out.push(',');
    out.push_str(&(seed as u32).to_string());
    out.push(',');
    bytes(out, &encode_bytes(value.as_bytes(), seed));
    out.push_str("},");
}

fn shuffled_order(len: usize, seed: u64) -> Vec<usize> {
    let mut order = (0..len).collect::<Vec<_>>();
    let mut rng = Prng::new(seed ^ 0xc011_57ab);
    for index in (1..order.len()).rev() {
        let swap = rng.range(index + 1);
        order.swap(index, swap);
    }
    order
}

fn item_seed(seed: u64, index: usize) -> u64 {
    seed ^ ((index as u64 + 1) * 0x9e37)
}

fn number_text(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        value.to_string()
    }
}
