use ferret_crypto::{encode_bytes, Prng};
use ferret_ir::{Capture, Chunk, Const};
use ferret_output::{ConstantLayout, NumberEncoder};

use crate::emit::lists::{bytes, words};
use crate::emit::opcodes::OpcodePlan;
use crate::emit::pack::encoded_words;

pub(super) fn constants(
    out: &mut String,
    constants: &[Const],
    seed: u64,
    opcodes: &OpcodePlan,
    layout: ConstantLayout,
    numbers: &mut NumberEncoder,
) {
    let order = shuffled_order(constants.len(), seed);
    let mut map = vec![0usize; constants.len()];
    for (slot, index) in order.iter().copied().enumerate() {
        map[index] = slot + 1;
    }
    let mut rows = String::new();
    rows.push('{');
    for index in order {
        constant(
            &mut rows,
            &constants[index],
            item_seed(seed, index),
            opcodes,
            layout,
            numbers,
        );
    }
    rows.push('}');
    let mut map_text = String::new();
    map_text.push('{');
    for slot in map {
        map_text.push_str(&numbers.usize(slot));
        map_text.push(',');
    }
    map_text.push('}');
    out.push('{');
    keyed(out, layout.rows, &rows, numbers);
    keyed(out, layout.map, &map_text, numbers);
    keyed(out, layout.cache, "{}", numbers);
    out.push('}');
}

fn constant(
    out: &mut String,
    constant: &Const,
    seed: u64,
    opcodes: &OpcodePlan,
    layout: ConstantLayout,
    numbers: &mut NumberEncoder,
) {
    match constant {
        Const::Nil => {
            out.push('{');
            out.push_str(&numbers.u8(0));
            out.push_str("},");
        }
        Const::Bool(value) => {
            out.push('{');
            out.push_str(&numbers.u8(1));
            out.push(',');
            out.push_str(&numbers.u8(u8::from(*value)));
            out.push_str("},");
        }
        Const::Number(value) => protected(out, 2, &number_text(*value), seed, numbers),
        Const::String(value) => protected(out, 3, value, seed, numbers),
        Const::Function { chunk, captures } => {
            function_const(out, chunk, captures, seed, opcodes, layout, numbers)
        }
    }
}

fn function_const(
    out: &mut String,
    chunk: &Chunk,
    captures: &[Capture],
    seed: u64,
    opcodes: &OpcodePlan,
    layout: ConstantLayout,
    numbers: &mut NumberEncoder,
) {
    let (encoded, stream_seed) = encoded_words(chunk, opcodes, seed, 0xf17e_f00d);
    out.push_str("{4,");
    out.push_str(&numbers.u32(stream_seed as u32));
    out.push(',');
    words(out, &encoded, numbers);
    out.push(',');
    constants(out, &chunk.constants, stream_seed, opcodes, layout, numbers);
    out.push(',');
    out.push_str(&numbers.u16(chunk.params));
    out.push(',');
    capture_list(out, captures, numbers);
    out.push_str("},");
}

fn keyed(out: &mut String, slot: usize, value: &str, numbers: &mut NumberEncoder) {
    out.push('[');
    out.push_str(&numbers.usize(slot));
    out.push_str("]=");
    out.push_str(value);
    out.push(',');
}

fn capture_list(out: &mut String, captures: &[Capture], numbers: &mut NumberEncoder) {
    out.push('{');
    for capture in captures {
        match capture {
            Capture::Local(reg) => capture_item(out, 0, *reg, numbers),
            Capture::Upvalue(index) => capture_item(out, 1, *index, numbers),
        }
    }
    out.push('}');
}

fn capture_item(out: &mut String, tag: u8, value: u16, numbers: &mut NumberEncoder) {
    out.push('{');
    out.push_str(&numbers.u8(tag));
    out.push(',');
    out.push_str(&numbers.u16(value));
    out.push_str("},");
}

fn protected(out: &mut String, tag: u8, value: &str, seed: u64, numbers: &mut NumberEncoder) {
    out.push('{');
    out.push_str(&numbers.u8(tag));
    out.push(',');
    out.push_str(&numbers.u32(seed as u32));
    out.push(',');
    bytes(out, &encode_bytes(value.as_bytes(), seed), numbers);
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
