#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: Kind,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    Ident(String),
    Number(String),
    String(String),
    Symbol(&'static str),
    Eof,
}

impl Kind {
    pub fn symbol(&self, value: &str) -> bool {
        matches!(self, Self::Symbol(symbol) if *symbol == value)
    }

    pub fn ident(&self, value: &str) -> bool {
        matches!(self, Self::Ident(ident) if ident == value)
    }
}
