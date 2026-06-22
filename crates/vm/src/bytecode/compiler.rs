mod calls;
mod captures;
mod expr;
mod patterns;
mod scope;
mod stmt;
mod superblocks;
mod table;

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
    let mut chunk = Chunk {
        constants: compiler.constants,
        instructions: compiler.instructions,
        registers: compiler.max_reg,
        params: 0,
    };
    superblocks::apply(&mut chunk)?;
    Ok(CompileReport { chunk })
}

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
    reg_floor: u16,
}

impl Default for Compiler {
    fn default() -> Self {
        Self {
            constants: Vec::new(),
            instructions: Vec::new(),
            locals: BTreeMap::new(),
            upvalues: BTreeMap::new(),
            breaks: Vec::new(),
            labels: BTreeMap::new(),
            gotos: Vec::new(),
            next_reg: 1,
            max_reg: 0,
            reg_floor: 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Binding {
    Local(u16),
    Cell(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EnvBinding {
    Local(Binding),
    Upvalue(u16),
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
            self.release_temps();
        }
        Ok(())
    }

    fn stmt(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Local { names, values } => {
                if names.len() == 1
                    && matches!(values.first(), Some(ferret_ir::Expr::Function { .. }))
                {
                    self.define_locals(names);
                    if let Some(value) = values.first() {
                        let src = self.expr(value)?;
                        let binding = self.locals[&names[0]];
                        self.write_binding(binding, src);
                    }
                    return Ok(());
                }

                let start = self.reserve(names.len() as u16);
                let regs = (0..names.len())
                    .map(|offset| start + offset as u16)
                    .collect::<Vec<_>>();
                let expands_tail = names.len() > values.len()
                    && matches!(
                        values.last(),
                        Some(ferret_ir::Expr::Call { .. } | ferret_ir::Expr::VarArgs)
                    );
                if expands_tail {
                    if let Some(ferret_ir::Expr::Call { callee, args }) = values.last() {
                        let fixed_count = values.len() - 1;
                        for (index, value) in values[..fixed_count].iter().enumerate() {
                            self.expr_into(regs[index], value)?;
                        }
                        self.call_n_into(
                            callee,
                            args,
                            regs[fixed_count],
                            (names.len() - fixed_count) as u16,
                        )?;
                    } else if matches!(values.last(), Some(ferret_ir::Expr::VarArgs)) {
                        let fixed_count = values.len() - 1;
                        for (index, value) in values[..fixed_count].iter().enumerate() {
                            self.expr_into(regs[index], value)?;
                        }
                        self.emit(
                            Op::VarArgN,
                            regs[fixed_count],
                            (names.len() - fixed_count) as u16,
                            0,
                        );
                    }
                } else {
                    for (index, dst) in regs.iter().copied().enumerate() {
                        if let Some(value) = values.get(index) {
                            self.expr_into(dst, value)?;
                        } else {
                            let nil = self.nil()?;
                            self.emit(Op::Move, dst, nil, 0);
                        }
                    }
                }
                self.bind_local_regs(names, &regs);
            }
            Stmt::Assign { targets, values } => {
                if self.special_assign(targets, values)? {
                    return Ok(());
                }
                let values = self.value_regs(values, targets.len())?;
                for (index, target) in targets.iter().enumerate() {
                    let src = match values.get(index).copied() {
                        Some(src) => src,
                        None => self.nil()?,
                    };
                    self.assign_target(target, src)?;
                }
            }
            Stmt::Block(body) => self.scoped_stmts(body)?,
            Stmt::Break => self.break_stmt()?,
            Stmt::Label(name) => self.label_stmt(name),
            Stmt::Goto(name) => self.goto_stmt(name)?,
            Stmt::Expr(expr) => {
                if let Stmt::Expr(ferret_ir::Expr::Call { callee, args }) = stmt {
                    if self.call_global_stmt(callee, args)? {
                        return Ok(());
                    }
                }
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
        self.max_reg = self.max_reg.max(reg);
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

    fn release_temps(&mut self) {
        self.next_reg = self.live_reg_limit();
    }

    fn live_reg_limit(&self) -> u16 {
        self.locals
            .values()
            .map(|binding| match binding {
                Binding::Local(reg) | Binding::Cell(reg) => reg + 1,
            })
            .max()
            .unwrap_or(1)
            .max(self.reg_floor)
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

    fn patch_c(&mut self, index: usize, target: u16) {
        self.instructions[index].c = target;
    }

    fn patch_breaks(&mut self, target: u16) {
        if let Some(breaks) = self.breaks.pop() {
            for jump in breaks {
                self.patch_a(jump, target);
            }
        }
    }
}
