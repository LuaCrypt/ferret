use ferret_ir::BinOp;
use ferret_util::{FerretError, Result};

use crate::lexer::Kind;

use super::{Parser, Token};

impl Parser {
    pub(super) fn check(&self, symbol: &str) -> bool {
        self.peek().kind.symbol(symbol)
    }

    pub(super) fn check_next(&self, symbol: &str) -> bool {
        self.peek_index(self.cursor + 1).kind.symbol(symbol)
    }

    pub(super) fn eat(&mut self, symbol: &str) -> bool {
        if self.check(symbol) {
            self.cursor += 1;
            true
        } else {
            false
        }
    }

    pub(super) fn expect(&mut self, symbol: &str) -> Result<()> {
        self.eat(symbol)
            .then_some(())
            .ok_or_else(|| self.err(&format!("expected '{symbol}'")))
    }

    pub(super) fn expect_ident(&mut self) -> Result<String> {
        match self.advance().kind {
            Kind::Ident(name) => Ok(name),
            _ => Err(self.err("expected identifier")),
        }
    }

    pub(super) fn expect_eof(&self) -> Result<()> {
        matches!(self.peek().kind, Kind::Eof)
            .then_some(())
            .ok_or_else(|| self.err("expected end of file"))
    }

    pub(super) fn at_any(&self, terms: &[&str]) -> bool {
        terms.iter().any(|term| {
            (*term == "eof" && matches!(self.peek().kind, Kind::Eof)) || self.check(term)
        })
    }

    pub(super) fn advance(&mut self) -> Token {
        let token = self.peek_index(self.cursor).clone();
        self.cursor += 1;
        token
    }

    pub(super) fn peek(&self) -> &Token {
        self.peek_index(self.cursor)
    }

    fn peek_index(&self, index: usize) -> &Token {
        self.tokens
            .get(index)
            .unwrap_or_else(|| self.tokens.last().expect("lexer always emits eof"))
    }

    pub(super) fn err(&self, message: &str) -> FerretError {
        FerretError::Parse(format!("{message} at line {}", self.peek().line))
    }

    pub(super) fn unsupported(&self, message: &str) -> FerretError {
        FerretError::Unsupported(format!("{message} at line {}", self.peek().line))
    }

    pub(super) fn binary_op(&self) -> Option<(BinOp, u8, u8)> {
        let op = match &self.peek().kind {
            Kind::Symbol("or") => (BinOp::Or, 1, 2),
            Kind::Symbol("and") => (BinOp::And, 3, 4),
            Kind::Symbol("==") => (BinOp::Eq, 5, 6),
            Kind::Symbol("~=") => (BinOp::Ne, 5, 6),
            Kind::Symbol("<") => (BinOp::Lt, 5, 6),
            Kind::Symbol("<=") => (BinOp::Le, 5, 6),
            Kind::Symbol(">") => (BinOp::Gt, 5, 6),
            Kind::Symbol(">=") => (BinOp::Ge, 5, 6),
            Kind::Symbol("|") => (BinOp::BitOr, 6, 7),
            Kind::Symbol("~") => (BinOp::BitXor, 8, 9),
            Kind::Symbol("&") => (BinOp::BitAnd, 10, 11),
            Kind::Symbol("<<") => (BinOp::Shl, 12, 13),
            Kind::Symbol(">>") => (BinOp::Shr, 12, 13),
            Kind::Symbol("+") => (BinOp::Add, 14, 15),
            Kind::Symbol("-") => (BinOp::Sub, 14, 15),
            Kind::Symbol("*") => (BinOp::Mul, 16, 17),
            Kind::Symbol("/") => (BinOp::Div, 16, 17),
            Kind::Symbol("//") => (BinOp::FloorDiv, 16, 17),
            Kind::Symbol("%") => (BinOp::Mod, 16, 17),
            Kind::Symbol("..") => (BinOp::Concat, 6, 5),
            Kind::Symbol("^") => (BinOp::Pow, 19, 18),
            _ => return None,
        };
        Some(op)
    }
}
