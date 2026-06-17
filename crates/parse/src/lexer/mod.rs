pub mod token;

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
                '-' if self.peek_next() == Some('-') => self.skip_comment(),
                '\'' | '"' => tokens.push(self.string()?),
                ch if ch.is_ascii_digit() => tokens.push(self.number()?),
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

    fn skip_comment(&mut self) {
        while let Some(ch) = self.peek() {
            self.cursor += 1;
            if ch == '\n' {
                self.line += 1;
                break;
            }
        }
    }

    fn string(&mut self) -> Result<Token> {
        let quote = self.advance().unwrap();
        let line = self.line;
        let mut out = String::new();
        while let Some(ch) = self.advance() {
            if ch == quote {
                return Ok(Token {
                    kind: Kind::String(out),
                    line,
                });
            }
            if ch == '\\' {
                out.push(match self.advance() {
                    Some('n') => '\n',
                    Some('t') => '\t',
                    Some('r') => '\r',
                    Some('\\') => '\\',
                    Some('"') => '"',
                    Some('\'') => '\'',
                    Some(other) => other,
                    None => return Err(self.err("unterminated escape")),
                });
            } else {
                out.push(ch);
            }
        }
        Err(self.err("unterminated string"))
    }

    fn number(&mut self) -> Result<Token> {
        let line = self.line;
        let start = self.cursor;
        while matches!(self.peek(), Some(ch) if ch.is_ascii_digit() || ch == '.') {
            self.cursor += 1;
        }
        let value = self.chars[start..self.cursor].iter().collect::<String>();
        let parsed = value
            .parse::<f64>()
            .map_err(|_| self.err("invalid number literal"))?;
        Ok(Token {
            kind: Kind::Number(parsed),
            line,
        })
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
