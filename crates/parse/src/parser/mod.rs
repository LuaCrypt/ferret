mod expr;
mod support;

use ferret_ir::{Expr, Program, Stmt};
use ferret_util::Result;

use crate::lexer::Token;

pub struct Parser {
    pub(super) tokens: Vec<Token>,
    pub(super) cursor: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, cursor: 0 }
    }

    pub fn parse_program(mut self) -> Result<Program> {
        let body = self.block(&["eof"])?;
        self.expect_eof()?;
        Ok(Program { body })
    }

    fn block(&mut self, terminators: &[&str]) -> Result<Vec<Stmt>> {
        let mut out = Vec::new();
        while !self.at_any(terminators) {
            if self.eat(";") {
                continue;
            }
            out.push(self.stmt()?);
        }
        Ok(out)
    }

    fn stmt(&mut self) -> Result<Stmt> {
        if self.eat("local") {
            self.local_stmt()
        } else if self.eat("if") {
            self.if_stmt()
        } else if self.eat("while") {
            self.while_stmt()
        } else if self.eat("repeat") {
            self.repeat_stmt()
        } else if self.eat("for") {
            self.for_stmt()
        } else if self.eat("break") {
            Ok(Stmt::Break)
        } else if self.eat("goto") {
            Ok(Stmt::Goto(self.expect_ident()?))
        } else if self.eat("::") {
            let name = self.expect_ident()?;
            self.expect("::")?;
            Ok(Stmt::Label(name))
        } else if self.eat("return") {
            self.return_stmt()
        } else if self.eat("do") {
            let body = self.block(&["end"])?;
            self.expect("end")?;
            Ok(Stmt::Block(body))
        } else if self.eat("function") {
            self.function_stmt()
        } else {
            self.assign_or_call()
        }
    }

    fn local_stmt(&mut self) -> Result<Stmt> {
        if self.eat("function") {
            let name = self.expect_ident()?;
            return Ok(Stmt::Local {
                names: vec![name],
                values: vec![self.function_expr()?],
            });
        }
        let names = self.local_names()?;
        let values = if self.eat("=") {
            self.expr_list()?
        } else {
            Vec::new()
        };
        Ok(Stmt::Local { names, values })
    }

    fn if_stmt(&mut self) -> Result<Stmt> {
        let cond = self.expr(0)?;
        self.expect("then")?;
        let then_body = self.block(&["elseif", "else", "end"])?;
        if self.eat("elseif") {
            return Ok(Stmt::If {
                cond,
                then_body,
                else_body: vec![self.if_stmt()?],
            });
        }
        let else_body = if self.eat("else") {
            self.block(&["end"])?
        } else {
            Vec::new()
        };
        self.expect("end")?;
        Ok(Stmt::If {
            cond,
            then_body,
            else_body,
        })
    }

    fn while_stmt(&mut self) -> Result<Stmt> {
        let cond = self.expr(0)?;
        self.expect("do")?;
        let body = self.block(&["end"])?;
        self.expect("end")?;
        Ok(Stmt::While { cond, body })
    }

    fn repeat_stmt(&mut self) -> Result<Stmt> {
        let body = self.block(&["until"])?;
        self.expect("until")?;
        let cond = self.expr(0)?;
        Ok(Stmt::Repeat { body, cond })
    }

    fn for_stmt(&mut self) -> Result<Stmt> {
        let name = self.expect_ident()?;
        if self.eat(",") || self.check("in") {
            return self.generic_for(name);
        }
        self.expect("=")?;
        let start = self.expr(0)?;
        self.expect(",")?;
        let end = self.expr(0)?;
        let step = if self.eat(",") {
            self.expr(0)?
        } else {
            Expr::Number(1.0)
        };
        self.expect("do")?;
        let body = self.block(&["end"])?;
        self.expect("end")?;
        Ok(Stmt::NumericFor {
            name,
            start,
            end,
            step,
            body,
        })
    }

    fn generic_for(&mut self, first: String) -> Result<Stmt> {
        let mut names = vec![first];
        while !self.eat("in") {
            names.push(self.expect_ident()?);
            if !self.eat(",") {
                self.expect("in")?;
                break;
            }
        }
        let iter = self.expr_list()?;
        self.expect("do")?;
        let body = self.block(&["end"])?;
        self.expect("end")?;
        Ok(Stmt::GenericFor { names, iter, body })
    }

    fn function_stmt(&mut self) -> Result<Stmt> {
        let mut target = Expr::Var(self.expect_ident()?);
        while self.eat(".") {
            target = Expr::Index {
                table: Box::new(target),
                key: Box::new(Expr::String(self.expect_ident()?)),
            };
        }
        let value = if self.eat(":") {
            target = Expr::Index {
                table: Box::new(target),
                key: Box::new(Expr::String(self.expect_ident()?)),
            };
            self.function_expr_with_self()?
        } else {
            self.function_expr()?
        };
        Ok(Stmt::Assign {
            targets: vec![target],
            values: vec![value],
        })
    }

    fn return_stmt(&mut self) -> Result<Stmt> {
        if self.at_any(&["end", "else", "until", "eof"]) {
            return Ok(Stmt::Return(Vec::new()));
        }
        let values = self.expr_list()?;
        Ok(Stmt::Return(values))
    }

    fn assign_or_call(&mut self) -> Result<Stmt> {
        let first = self.prefix_expr()?;
        let mut targets = vec![first];
        while self.eat(",") {
            targets.push(self.prefix_expr()?);
        }
        if self.eat("=") {
            Ok(Stmt::Assign {
                targets,
                values: self.expr_list()?,
            })
        } else if targets.len() == 1 {
            Ok(Stmt::Expr(targets.remove(0)))
        } else {
            Err(self.err("expected '='"))
        }
    }

    fn local_names(&mut self) -> Result<Vec<String>> {
        let mut names = vec![self.local_name()?];
        while self.eat(",") {
            names.push(self.local_name()?);
        }
        Ok(names)
    }

    fn local_name(&mut self) -> Result<String> {
        let name = self.expect_ident()?;
        if self.eat("<") {
            let attr = self.expect_ident()?;
            self.expect(">")?;
            if attr == "close" {
                return Err(self.unsupported("to-be-closed variables are not supported yet"));
            }
            if attr != "const" {
                return Err(self.unsupported("unknown local attribute"));
            }
        }
        Ok(name)
    }
}
