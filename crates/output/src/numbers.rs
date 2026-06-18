use ferret_crypto::Prng;

pub struct NumberEncoder {
    rng: Prng,
}

impl NumberEncoder {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: Prng::new(seed ^ 0x51de_ca1f),
        }
    }

    pub fn u8(&mut self, value: u8) -> String {
        self.u32(u32::from(value))
    }

    pub fn u16(&mut self, value: u16) -> String {
        self.u32(u32::from(value))
    }

    pub fn u32(&mut self, value: u32) -> String {
        if self.rng.range(2) == 0 {
            value.to_string()
        } else {
            format!("0x{value:x}")
        }
    }

    pub fn usize(&mut self, value: usize) -> String {
        self.u32(value as u32)
    }

    pub fn noise(&mut self) -> u32 {
        self.rng.next_u32()
    }
}

pub fn rewrite_number_literals(code: &str, numbers: &mut NumberEncoder) -> String {
    let mut out = String::with_capacity(code.len());
    let mut chars = code.char_indices().peekable();
    while let Some((start, ch)) = chars.peek().copied() {
        if ch == '\'' || ch == '"' {
            copy_string(&mut chars, &mut out, ch);
        } else if is_ident_start(ch) {
            copy_ident(code, &mut chars, &mut out, start);
        } else if ch.is_ascii_digit() {
            let literal = take_number(code, &mut chars, start);
            if let Some(value) = parse_number(literal) {
                out.push_str(&numbers.u32(value));
            } else {
                out.push_str(literal);
            }
        } else {
            out.push(ch);
            chars.next();
        }
    }
    out
}

fn copy_ident<'a>(
    code: &'a str,
    chars: &mut std::iter::Peekable<std::str::CharIndices<'a>>,
    out: &mut String,
    start: usize,
) {
    let mut end = start;
    while let Some((index, ch)) = chars.peek().copied() {
        if !is_ident_next(ch) {
            break;
        }
        chars.next();
        end = index + ch.len_utf8();
    }
    out.push_str(&code[start..end]);
}

fn take_number<'a>(
    code: &'a str,
    chars: &mut std::iter::Peekable<std::str::CharIndices<'a>>,
    start: usize,
) -> &'a str {
    let mut end = start;
    while let Some((index, ch)) = chars.peek().copied() {
        if !ch.is_ascii_hexdigit() && ch != 'x' && ch != 'X' {
            break;
        }
        chars.next();
        end = index + ch.len_utf8();
    }
    &code[start..end]
}

fn parse_number(literal: &str) -> Option<u32> {
    if let Some(hex) = literal
        .strip_prefix("0x")
        .or_else(|| literal.strip_prefix("0X"))
    {
        u32::from_str_radix(hex, 16).ok()
    } else {
        literal.parse::<u32>().ok()
    }
}

fn copy_string(
    chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>,
    out: &mut String,
    quote: char,
) {
    let mut escaped = false;
    for (_, ch) in chars.by_ref() {
        out.push(ch);
        if escaped {
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == quote {
            break;
        }
    }
}

fn is_ident_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_ident_next(ch: char) -> bool {
    is_ident_start(ch) || ch.is_ascii_digit()
}
