use ferret_ir::{Expr, UnOp};
use ferret_util::Result;

use crate::lexer::Kind;

use super::Parser;

impl Parser {
    pub(super) fn expr_list(&mut self) -> Result<Vec<Expr>> {
        let mut values = vec![self.expr(0)?];
        while self.eat(",") {
            values.push(self.expr(0)?);
        }
        Ok(values)
    }

    pub(super) fn expr(&mut self, min_bp: u8) -> Result<Expr> {
        let mut left = self.unary()?;
        while let Some((op, left_bp, right_bp)) = self.binary_op() {
            if left_bp < min_bp {
                break;
            }
            self.cursor += 1;
            let right = self.expr(right_bp)?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn unary(&mut self) -> Result<Expr> {
        if self.eat("-") {
            Ok(Expr::Unary {
                op: UnOp::Neg,
                expr: Box::new(self.expr(18)?),
            })
        } else if self.eat("not") {
            Ok(Expr::Unary {
                op: UnOp::Not,
                expr: Box::new(self.expr(18)?),
            })
        } else if self.eat("#") {
            Ok(Expr::Unary {
                op: UnOp::Len,
                expr: Box::new(self.expr(18)?),
            })
        } else if self.eat("~") {
            Ok(Expr::Unary {
                op: UnOp::BitNot,
                expr: Box::new(self.expr(18)?),
            })
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Expr> {
        match self.advance().kind.clone() {
            Kind::Number(value) => Ok(Expr::Number(value)),
            Kind::String(value) => Ok(Expr::String(value)),
            Kind::Symbol("nil") => Ok(Expr::Nil),
            Kind::Symbol("true") => Ok(Expr::Bool(true)),
            Kind::Symbol("false") => Ok(Expr::Bool(false)),
            Kind::Symbol("...") => Ok(Expr::VarArgs),
            Kind::Symbol("function") => self.function_body(None),
            Kind::Ident(name) => self.finish_prefix(Expr::Var(name)),
            Kind::Symbol("(") => {
                let expr = self.expr(0)?;
                self.expect(")")?;
                self.finish_prefix(expr)
            }
            Kind::Symbol("{") => self.table(),
            _ => Err(self.err("expected expression")),
        }
    }

    pub(super) fn prefix_expr(&mut self) -> Result<Expr> {
        match self.advance().kind.clone() {
            Kind::Ident(name) => self.finish_prefix(Expr::Var(name)),
            Kind::Symbol("(") => {
                let expr = self.expr(0)?;
                self.expect(")")?;
                self.finish_prefix(expr)
            }
            _ => Err(self.err("expected assignment target or function call")),
        }
    }

    pub(super) fn function_expr(&mut self) -> Result<Expr> {
        self.function_body(None)
    }

    pub(super) fn function_expr_with_self(&mut self) -> Result<Expr> {
        self.function_body(Some("self".to_string()))
    }

    fn function_body(&mut self, first_param: Option<String>) -> Result<Expr> {
        let (mut params, vararg) = self.params()?;
        if let Some(first_param) = first_param {
            params.insert(0, first_param);
        }
        let body = self.block(&["end"])?;
        self.expect("end")?;
        Ok(Expr::Function {
            params,
            vararg,
            body,
        })
    }

    fn params(&mut self) -> Result<(Vec<String>, bool)> {
        self.expect("(")?;
        if self.eat(")") {
            return Ok((Vec::new(), false));
        }
        let mut params = Vec::new();
        loop {
            if self.eat("...") {
                self.expect(")")?;
                return Ok((params, true));
            }
            params.push(self.expect_ident()?);
            if self.eat(")") {
                return Ok((params, false));
            }
            self.expect(",")?;
        }
    }

    fn finish_prefix(&mut self, mut expr: Expr) -> Result<Expr> {
        loop {
            if self.eat("(") {
                expr = self.call_with_open_paren(expr)?;
            } else if self.eat(".") {
                expr = Expr::Index {
                    table: Box::new(expr),
                    key: Box::new(Expr::String(self.expect_ident()?)),
                };
            } else if self.eat(":") {
                expr = self.method_call(expr)?;
            } else if self.eat("[") {
                let key = self.expr(0)?;
                self.expect("]")?;
                expr = Expr::Index {
                    table: Box::new(expr),
                    key: Box::new(key),
                };
            } else if let Some(args) = self.single_call_arg()? {
                expr = Expr::Call {
                    callee: Box::new(expr),
                    args,
                };
            } else {
                return Ok(expr);
            }
        }
    }

    fn call_with_open_paren(&mut self, callee: Expr) -> Result<Expr> {
        let args = if self.eat(")") {
            Vec::new()
        } else {
            let args = self.expr_list()?;
            self.expect(")")?;
            args
        };
        Ok(Expr::Call {
            callee: Box::new(callee),
            args,
        })
    }

    fn method_call(&mut self, receiver: Expr) -> Result<Expr> {
        let callee = Expr::Index {
            table: Box::new(receiver.clone()),
            key: Box::new(Expr::String(self.expect_ident()?)),
        };
        let mut args = vec![receiver];
        if self.eat("(") {
            if self.eat(")") {
                return Ok(Expr::Call {
                    callee: Box::new(callee),
                    args,
                });
            }
            args.extend(self.expr_list()?);
            self.expect(")")?;
        } else if let Some(extra) = self.single_call_arg()? {
            args.extend(extra);
        } else {
            self.expect("(")?;
        }
        Ok(Expr::Call {
            callee: Box::new(callee),
            args,
        })
    }

    fn single_call_arg(&mut self) -> Result<Option<Vec<Expr>>> {
        match self.peek().kind.clone() {
            Kind::String(value) => {
                self.advance();
                Ok(Some(vec![Expr::String(value)]))
            }
            Kind::Symbol("{") => {
                self.advance();
                Ok(Some(vec![self.table()?]))
            }
            _ => Ok(None),
        }
    }

    fn table(&mut self) -> Result<Expr> {
        let mut fields = Vec::new();
        while !self.eat("}") {
            fields.push(self.table_field()?);
            if !self.eat(",") && !self.eat(";") {
                self.expect("}")?;
                break;
            }
        }
        Ok(Expr::Table(fields))
    }

    fn table_field(&mut self) -> Result<(Option<Expr>, Expr)> {
        if self.check("[") {
            self.expect("[")?;
            let key = self.expr(0)?;
            self.expect("]")?;
            self.expect("=")?;
            return Ok((Some(key), self.expr(0)?));
        }
        if matches!(self.peek().kind, Kind::Ident(_)) && self.check_next("=") {
            let key = Expr::String(self.expect_ident()?);
            self.expect("=")?;
            return Ok((Some(key), self.expr(0)?));
        }
        Ok((None, self.expr(0)?))
    }
}
