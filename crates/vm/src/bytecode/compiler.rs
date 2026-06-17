mod calls;
mod captures;
mod expr;
mod stmt;

use std::collections::{BTreeMap, BTreeSet};

use ferret_ir::{Chunk, Const, Instr, Op, Program, Stmt};
use ferret_util::Result;

use super::support::to_u16;

#[derive(Debug, Clone, PartialEq)]
pub struct CompileReport {
    pub chunk: Chunk,
}

pub fn compile(program: &Program) -> Result<CompileReport> {
    let mut compiler = Compiler::default();
    compiler.stmts(&program.body)?;
    compiler.finish_gotos()?;
    compiler.emit(Op::Halt, 0, 0, 0);
    Ok(CompileReport {
        chunk: Chunk {
            constants: compiler.constants,
            instructions: compiler.instructions,
            registers: compiler.max_reg,
            params: 0,
        },
    })
}

#[derive(Default)]
struct Compiler {
    constants: Vec<Const>,
    instructions: Vec<Instr>,
    locals: BTreeMap<String, Binding>,
    upvalues: BTreeMap<String, u16>,
    breaks: Vec<Vec<usize>>,
    labels: BTreeMap<String, u16>,
    gotos: Vec<(String, usize)>,
    next_reg: u16,
    max_reg: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Binding {
    Local(u16),
    Cell(u16),
}

impl Compiler {
    pub(super) fn with_params(params: &[String], upvalues: BTreeMap<String, u16>) -> Self {
        let mut compiler = Self {
            upvalues,
            ..Self::default()
        };
        for param in params {
            let reg = compiler.alloc();
            compiler.locals.insert(param.clone(), Binding::Local(reg));
        }
        compiler
    }

    pub(super) fn capture_names(&self) -> BTreeSet<String> {
        self.locals
            .keys()
            .chain(self.upvalues.keys())
            .cloned()
            .collect()
    }

    fn stmts(&mut self, stmts: &[Stmt]) -> Result<()> {
        for stmt in stmts {
            self.stmt(stmt)?;
        }
        Ok(())
    }

    fn stmt(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Local { names, values } => {
                self.define_locals(names);
                let values = self.value_regs(values, names.len())?;
                for (index, name) in names.iter().enumerate() {
                    let src = match values.get(index).copied() {
                        Some(src) => src,
                        None => self.nil()?,
                    };
                    let binding = self.locals[name];
                    self.write_binding(binding, src);
                }
            }
            Stmt::Assign { targets, values } => {
                let values = self.value_regs(values, targets.len())?;
                for (index, target) in targets.iter().enumerate() {
                    let src = match values.get(index).copied() {
                        Some(src) => src,
                        None => self.nil()?,
                    };
                    self.assign_target(target, src)?;
                }
            }
            Stmt::Block(body) => {
                self.stmts(body)?;
            }
            Stmt::Break => self.break_stmt()?,
            Stmt::Label(name) => self.label_stmt(name),
            Stmt::Goto(name) => self.goto_stmt(name)?,
            Stmt::Expr(expr) => {
                self.expr(expr)?;
            }
            Stmt::If {
                cond,
                then_body,
                else_body,
            } => self.if_stmt(cond, then_body, else_body)?,
            Stmt::While { cond, body } => self.while_stmt(cond, body)?,
            Stmt::Repeat { body, cond } => self.repeat_stmt(body, cond)?,
            Stmt::NumericFor {
                name,
                start,
                end,
                step,
                body,
            } => self.for_stmt(name, start, end, step, body)?,
            Stmt::GenericFor { names, iter, body } => self.generic_for(names, iter, body)?,
            Stmt::Return(values) => self.return_stmt(values)?,
        }
        Ok(())
    }

    fn constant(&mut self, value: Const) -> Result<u16> {
        if let Some(index) = self
            .constants
            .iter()
            .position(|existing| *existing == value)
        {
            return to_u16(index);
        }
        let index = self.constants.len();
        self.constants.push(value);
        to_u16(index)
    }

    fn alloc(&mut self) -> u16 {
        let reg = self.next_reg;
        self.next_reg = self.next_reg.saturating_add(1);
        self.max_reg = self.max_reg.max(self.next_reg);
        reg
    }

    fn reserve(&mut self, count: u16) -> u16 {
        let start = self.next_reg;
        for _ in 0..count {
            self.alloc();
        }
        start
    }

    fn reserve_to(&mut self, end: u16) {
        while self.next_reg < end {
            self.alloc();
        }
    }

    fn emit(&mut self, op: Op, a: u16, b: u16, c: u16) -> usize {
        let index = self.instructions.len();
        self.instructions.push(Instr::new(op, a, b, c));
        index
    }

    fn patch_a(&mut self, index: usize, target: u16) {
        self.instructions[index].a = target;
    }

    fn patch_b(&mut self, index: usize, target: u16) {
        self.instructions[index].b = target;
    }

    fn patch_breaks(&mut self, target: u16) {
        if let Some(breaks) = self.breaks.pop() {
            for jump in breaks {
                self.patch_a(jump, target);
            }
        }
    }
}
