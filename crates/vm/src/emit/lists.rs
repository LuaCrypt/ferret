use ferret_output::NumberEncoder;

pub(super) fn words(out: &mut String, values: &[u32], numbers: &mut NumberEncoder) {
    out.push('{');
    for value in values {
        out.push_str(&numbers.u32(*value));
        out.push(',');
    }
    out.push('}');
}

pub(super) fn bytes(out: &mut String, values: &[u8], numbers: &mut NumberEncoder) {
    out.push('{');
    for value in values {
        out.push_str(&numbers.u8(*value));
        out.push(',');
    }
    out.push('}');
}
