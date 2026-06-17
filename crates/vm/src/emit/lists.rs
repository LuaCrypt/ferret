pub(super) fn words(out: &mut String, values: &[u32]) {
    out.push('{');
    for value in values {
        out.push_str(&value.to_string());
        out.push(',');
    }
    out.push('}');
}

pub(super) fn bytes(out: &mut String, values: &[u8]) {
    out.push('{');
    for value in values {
        out.push_str(&value.to_string());
        out.push(',');
    }
    out.push('}');
}
