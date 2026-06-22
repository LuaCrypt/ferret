pub mod token;

mod long;
mod short_string;

use ferret_util::{FerretError, Result};
pub use token::{Kind, Token};

const KEYWORDS: &[&str] = &[
    "and", "break", "do", "else", "elseif", "end", "false", "for", "function", "goto", "if", "in",
    "local", "nil", "not", "or", "repeat", "return", "then", "true", "until", "while",
];

pub fn lex(source: &str) -> Result<Vec<Token>> {
    let mut lexer = Lexer::new(source);
    lexer.lex_all()
}

struct Lexer<'a> {
    chars: Vec<char>,
    cursor: usize,
    line: usize,
    _source: &'a str,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            chars: source.chars().collect(),
            cursor: 0,
            line: 1,
            _source: source,
        }
    }

    fn lex_all(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        while let Some(ch) = self.peek() {
            match ch {
                '\n' => {
                    self.line += 1;
                    self.cursor += 1;
                }
                ch if ch.is_whitespace() => self.cursor += 1,
                '-' if self.peek_next() == Some('-') => self.skip_comment()?,
                '[' if long::level(&self.chars, self.cursor).is_some() => {
                    tokens.push(self.long_string()?)
                }
                '\'' | '"' => tokens.push(short_string::lex(
                    &self.chars,
                    &mut self.cursor,
                    &mut self.line,
                )?),
                ch if ch.is_ascii_digit()
                    || (ch == '.'
                        && self.peek_next().is_some_and(|next| next.is_ascii_digit())) =>
                {
                    tokens.push(self.number())
                }
                ch if is_ident_start(ch) => tokens.push(self.ident()),
                _ => tokens.push(self.symbol()?),
            }
        }
        tokens.push(Token {
            kind: Kind::Eof,
            line: self.line,
        });
        Ok(tokens)
    }

    fn skip_comment(&mut self) -> Result<()> {
        self.cursor += 2;
        if long::level(&self.chars, self.cursor).is_some() {
            self.long_string()?;
            return Ok(());
        }
        while let Some(ch) = self.peek() {
            self.cursor += 1;
            if ch == '\n' {
                self.line += 1;
                break;
            }
        }
        Ok(())
    }

    fn long_string(&mut self) -> Result<Token> {
        let line = self.line;
        let level = long::level(&self.chars, self.cursor)
            .ok_or_else(|| self.err("expected long string"))?;
        self.cursor += level + 2;
        if self.peek() == Some('\n') {
            self.cursor += 1;
            self.line += 1;
        }
        let mut out = String::new();
        loop {
            let Some(ch) = self.peek() else {
                return Err(self.err("unterminated long string"));
            };
            if long::close(&self.chars, self.cursor, level) {
                self.cursor += level + 2;
                return Ok(Token {
                    kind: Kind::String(out),
                    line,
                });
            }
            self.cursor += 1;
            if ch == '\n' {
                self.line += 1;
            }
            out.push(ch);
        }
    }

    fn number(&mut self) -> Token {
        let line = self.line;
        let start = self.cursor;
        if self.peek() == Some('0') && matches!(self.peek_next(), Some('x' | 'X')) {
            self.cursor += 2;
            self.take_while(|ch| ch.is_ascii_hexdigit());
            if self.peek() == Some('.') {
                self.cursor += 1;
                self.take_while(|ch| ch.is_ascii_hexdigit());
            }
            if matches!(self.peek(), Some('p' | 'P')) {
                self.cursor += 1;
                if matches!(self.peek(), Some('+' | '-')) {
                    self.cursor += 1;
                }
                self.take_while(|ch| ch.is_ascii_digit());
            }
        } else {
            self.take_while(|ch| ch.is_ascii_digit());
            if self.peek() == Some('.') && self.peek_next().is_none_or(|next| next != '.') {
                self.cursor += 1;
                self.take_while(|ch| ch.is_ascii_digit());
            }
            if matches!(self.peek(), Some('e' | 'E')) {
                self.cursor += 1;
                if matches!(self.peek(), Some('+' | '-')) {
                    self.cursor += 1;
                }
                self.take_while(|ch| ch.is_ascii_digit());
            }
        }
        let value = self.chars[start..self.cursor].iter().collect::<String>();
        Token {
            kind: Kind::Number(value),
            line,
        }
    }

    fn ident(&mut self) -> Token {
        let line = self.line;
        let start = self.cursor;
        while matches!(self.peek(), Some(ch) if is_ident_continue(ch)) {
            self.cursor += 1;
        }
        let ident = self.chars[start..self.cursor].iter().collect::<String>();
        let kind = if KEYWORDS.contains(&ident.as_str()) {
            Kind::Symbol(Box::leak(ident.into_boxed_str()))
        } else {
            Kind::Ident(ident)
        };
        Token { kind, line }
    }

    fn symbol(&mut self) -> Result<Token> {
        let line = self.line;
        if self.peek() == Some('.')
            && self.peek_next() == Some('.')
            && self.chars.get(self.cursor + 2).copied() == Some('.')
        {
            self.cursor += 3;
            return Ok(Token {
                kind: Kind::Symbol("..."),
                line,
            });
        }
        let two = match (self.peek(), self.peek_next()) {
            (Some('='), Some('=')) => Some("=="),
            (Some('~'), Some('=')) => Some("~="),
            (Some('<'), Some('=')) => Some("<="),
            (Some('>'), Some('=')) => Some(">="),
            (Some('<'), Some('<')) => Some("<<"),
            (Some('>'), Some('>')) => Some(">>"),
            (Some('.'), Some('.')) => Some(".."),
            (Some('/'), Some('/')) => Some("//"),
            (Some(':'), Some(':')) => Some("::"),
            _ => None,
        };
        if let Some(symbol) = two {
            self.cursor += 2;
            return Ok(Token {
                kind: Kind::Symbol(symbol),
                line,
            });
        }
        let symbol = match self.advance().unwrap() {
            '+' => "+",
            '-' => "-",
            '*' => "*",
            '/' => "/",
            '&' => "&",
            '|' => "|",
            '%' => "%",
            '~' => "~",
            '^' => "^",
            '#' => "#",
            '=' => "=",
            '<' => "<",
            '>' => ">",
            '(' => "(",
            ')' => ")",
            '{' => "{",
            '}' => "}",
            '[' => "[",
            ']' => "]",
            ',' => ",",
            ';' => ";",
            ':' => ":",
            '.' => ".",
            other => return Err(self.err(&format!("unsupported character '{other}'"))),
        };
        Ok(Token {
            kind: Kind::Symbol(symbol),
            line,
        })
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.cursor += 1;
        Some(ch)
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.cursor).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.cursor + 1).copied()
    }

    fn take_while(&mut self, mut pred: impl FnMut(char) -> bool) {
        while matches!(self.peek(), Some(ch) if pred(ch)) {
            self.cursor += 1;
        }
    }

    fn err(&self, message: &str) -> FerretError {
        FerretError::Parse(format!("{message} at line {}", self.line))
    }
}

fn is_ident_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_ident_continue(ch: char) -> bool {
    is_ident_start(ch) || ch.is_ascii_digit()
}
