#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding {
    pub from: String,
    pub to: String,
}

pub fn rename_identifiers(code: &str, bindings: &[Binding]) -> String {
    let mut out = String::with_capacity(code.len());
    let mut chars = code.char_indices().peekable();
    while let Some((_, ch)) = chars.peek().copied() {
        if ch == '\'' || ch == '"' {
            copy_string(&mut chars, &mut out, ch);
            continue;
        }
        if is_ident_start(ch) {
            let start = chars.next().unwrap().0;
            let mut end = start + ch.len_utf8();
            while let Some((index, next)) = chars.peek().copied() {
                if !is_ident_next(next) {
                    break;
                }
                chars.next();
                end = index + next.len_utf8();
            }
            let ident = &code[start..end];
            if let Some(binding) = bindings.iter().find(|binding| binding.from == ident) {
                out.push_str(&binding.to);
            } else {
                out.push_str(ident);
            }
        } else {
            out.push(ch);
            chars.next();
        }
    }
    out
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
