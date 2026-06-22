use ferret_util::{FerretError, Result};

use super::{Kind, Token};

pub(super) fn lex(chars: &[char], cursor: &mut usize, line: &mut usize) -> Result<Token> {
    let quote = chars[*cursor];
    let start_line = *line;
    *cursor += 1;
    let mut out = String::new();
    while *cursor < chars.len() {
        let ch = take(chars, cursor).expect("bounds checked");
        if ch == quote {
            return Ok(Token {
                kind: Kind::String(out),
                line: start_line,
            });
        }
        if ch == '\n' {
            return Err(err(*line, "unterminated string"));
        }
        if ch == '\\' {
            if let Some(value) = escape(chars, cursor, line)? {
                out.push_str(&value);
            }
        } else {
            out.push(ch);
        }
    }
    Err(err(*line, "unterminated string"))
}

fn escape(chars: &[char], cursor: &mut usize, line: &mut usize) -> Result<Option<String>> {
    let ch = take(chars, cursor).ok_or_else(|| err(*line, "unterminated escape"))?;
    let value = match ch {
        'a' => "\x07".to_string(),
        'b' => "\x08".to_string(),
        'f' => "\x0c".to_string(),
        'n' => "\n".to_string(),
        'r' => "\r".to_string(),
        't' => "\t".to_string(),
        'v' => "\x0b".to_string(),
        '\\' => "\\".to_string(),
        '"' => "\"".to_string(),
        '\'' => "'".to_string(),
        '\n' => {
            *line += 1;
            "\n".to_string()
        }
        'z' => {
            skip_space(chars, cursor, line);
            return Ok(None);
        }
        'x' => byte_string(&[hex_byte(chars, cursor, line)?]),
        'u' => unicode_escape(chars, cursor, line)?,
        digit if digit.is_ascii_digit() => {
            byte_string(&[decimal_byte(digit, chars, cursor, line)?])
        }
        other => other.to_string(),
    };
    Ok(Some(value))
}

fn skip_space(chars: &[char], cursor: &mut usize, line: &mut usize) {
    while chars.get(*cursor).is_some_and(|ch| ch.is_whitespace()) {
        if chars[*cursor] == '\n' {
            *line += 1;
        }
        *cursor += 1;
    }
}

fn decimal_byte(first: char, chars: &[char], cursor: &mut usize, line: &mut usize) -> Result<u8> {
    let mut value = first.to_digit(10).unwrap();
    for _ in 0..2 {
        let Some(ch) = chars.get(*cursor).copied() else {
            break;
        };
        if !ch.is_ascii_digit() {
            break;
        }
        *cursor += 1;
        value = value * 10 + ch.to_digit(10).unwrap();
    }
    u8::try_from(value).map_err(|_| err(*line, "decimal escape too large"))
}

fn hex_byte(chars: &[char], cursor: &mut usize, line: &mut usize) -> Result<u8> {
    let hi = take_hex(chars, cursor).ok_or_else(|| err(*line, "invalid hex escape"))?;
    let lo = take_hex(chars, cursor).ok_or_else(|| err(*line, "invalid hex escape"))?;
    Ok(((hi << 4) | lo) as u8)
}

fn unicode_escape(chars: &[char], cursor: &mut usize, line: &mut usize) -> Result<String> {
    if take(chars, cursor) != Some('{') {
        return Err(err(*line, "invalid unicode escape"));
    }
    let mut value = 0u32;
    let mut digits = 0usize;
    loop {
        let Some(ch) = take(chars, cursor) else {
            return Err(err(*line, "unterminated unicode escape"));
        };
        if ch == '}' {
            break;
        }
        let Some(digit) = ch.to_digit(16) else {
            return Err(err(*line, "invalid unicode escape"));
        };
        value = value * 16 + digit;
        digits += 1;
    }
    if digits == 0 {
        return Err(err(*line, "empty unicode escape"));
    }
    if let Some(ch) = char::from_u32(value) {
        return Ok(ch.to_string());
    }
    utf8_bytes(value, *line).map(|bytes| byte_string(&bytes))
}

fn take_hex(chars: &[char], cursor: &mut usize) -> Option<u32> {
    let ch = chars.get(*cursor).copied()?;
    let value = ch.to_digit(16)?;
    *cursor += 1;
    Some(value)
}

fn take(chars: &[char], cursor: &mut usize) -> Option<char> {
    let ch = chars.get(*cursor).copied()?;
    *cursor += 1;
    Some(ch)
}

fn byte_string(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| char::from(*byte)).collect()
}

fn utf8_bytes(value: u32, line: usize) -> Result<Vec<u8>> {
    if value <= 0x7f {
        Ok(vec![value as u8])
    } else if value <= 0x7ff {
        Ok(vec![
            0xc0 | ((value >> 6) as u8),
            0x80 | ((value & 0x3f) as u8),
        ])
    } else if value <= 0xffff {
        Ok(vec![
            0xe0 | ((value >> 12) as u8),
            0x80 | (((value >> 6) & 0x3f) as u8),
            0x80 | ((value & 0x3f) as u8),
        ])
    } else if value <= 0x1f_ffff {
        Ok(vec![
            0xf0 | ((value >> 18) as u8),
            0x80 | (((value >> 12) & 0x3f) as u8),
            0x80 | (((value >> 6) & 0x3f) as u8),
            0x80 | ((value & 0x3f) as u8),
        ])
    } else if value <= 0x3ff_ffff {
        Ok(vec![
            0xf8 | ((value >> 24) as u8),
            0x80 | (((value >> 18) & 0x3f) as u8),
            0x80 | (((value >> 12) & 0x3f) as u8),
            0x80 | (((value >> 6) & 0x3f) as u8),
            0x80 | ((value & 0x3f) as u8),
        ])
    } else if value <= 0x7fff_ffff {
        Ok(vec![
            0xfc | (((value >> 30) & 0x01) as u8),
            0x80 | (((value >> 24) & 0x3f) as u8),
            0x80 | (((value >> 18) & 0x3f) as u8),
            0x80 | (((value >> 12) & 0x3f) as u8),
            0x80 | (((value >> 6) & 0x3f) as u8),
            0x80 | ((value & 0x3f) as u8),
        ])
    } else {
        Err(err(line, "unicode escape too large"))
    }
}

fn err(line: usize, message: &str) -> FerretError {
    FerretError::Parse(format!("{message} at line {line}"))
}
