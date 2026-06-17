use ferret_ir::{Expr, Op, Stmt};
use ferret_util::{FerretError, Result};

use super::{Binding, Compiler};

impl Compiler {
    pub(super) fn if_stmt(
        &mut self,
        cond: &Expr,
        then_body: &[Stmt],
        else_body: &[Stmt],
    ) -> Result<()> {
        let cond = self.expr(cond)?;
        let false_jump = self.emit(Op::JmpFalse, cond, 0, 0);
        self.stmts(then_body)?;
        let end_jump = self.emit(Op::Jmp, 0, 0, 0);
        let else_start = self.instructions.len() as u16;
        self.patch_b(false_jump, else_start);
        self.stmts(else_body)?;
        let end = self.instructions.len() as u16;
        self.patch_a(end_jump, end);
        Ok(())
    }

    pub(super) fn while_stmt(&mut self, cond: &Expr, body: &[Stmt]) -> Result<()> {
        let start = self.instructions.len() as u16;
        let cond = self.expr(cond)?;
        let end_jump = self.emit(Op::JmpFalse, cond, 0, 0);
        self.breaks.push(Vec::new());
        self.stmts(body)?;
        self.emit(Op::Jmp, start, 0, 0);
        let end = self.instructions.len() as u16;
        self.patch_b(end_jump, end);
        self.patch_breaks(end);
        Ok(())
    }

    pub(super) fn repeat_stmt(&mut self, body: &[Stmt], cond: &Expr) -> Result<()> {
        let start = self.instructions.len() as u16;
        self.breaks.push(Vec::new());
        self.stmts(body)?;
        let cond = self.expr(cond)?;
        self.emit(Op::JmpFalse, cond, start, 0);
        let end = self.instructions.len() as u16;
        self.patch_breaks(end);
        Ok(())
    }

    pub(super) fn for_stmt(
        &mut self,
        name: &str,
        start: &Expr,
        end: &Expr,
        step: &Expr,
        body: &[Stmt],
    ) -> Result<()> {
        let var = self.alloc();
        self.locals.insert(name.to_string(), Binding::Local(var));
        let start_reg = self.expr(start)?;
        self.emit(Op::Move, var, start_reg, 0);
        let end_reg = self.expr(end)?;
        let step_reg = self.expr(step)?;
        let loop_start = self.instructions.len() as u16;
        let cond = self.alloc();
        self.emit(Op::Le, cond, var, end_reg);
        let exit = self.emit(Op::JmpFalse, cond, 0, 0);
        self.breaks.push(Vec::new());
        self.stmts(body)?;
        self.emit(Op::Add, var, var, step_reg);
        self.emit(Op::Jmp, loop_start, 0, 0);
        let done = self.instructions.len() as u16;
        self.patch_b(exit, done);
        self.patch_breaks(done);
        Ok(())
    }

    pub(super) fn break_stmt(&mut self) -> Result<()> {
        if self.breaks.is_empty() {
            return Err(FerretError::Unsupported("break outside loop".to_string()));
        }
        let jump = self.emit(Op::Jmp, 0, 0, 0);
        self.breaks.last_mut().expect("loop checked").push(jump);
        Ok(())
    }

    pub(super) fn label_stmt(&mut self, name: &str) {
        let target = self.instructions.len() as u16;
        self.labels.insert(name.to_string(), target);
    }

    pub(super) fn goto_stmt(&mut self, name: &str) -> Result<()> {
        let jump = self.emit(Op::Jmp, 0, 0, 0);
        if let Some(target) = self.labels.get(name).copied() {
            self.patch_a(jump, target);
        } else {
            self.gotos.push((name.to_string(), jump));
        }
        Ok(())
    }

    pub(super) fn finish_gotos(&mut self) -> Result<()> {
        let pending = std::mem::take(&mut self.gotos);
        for (name, jump) in pending {
            let Some(target) = self.labels.get(&name).copied() else {
                return Err(FerretError::Unsupported(format!(
                    "unknown goto label '{name}'"
                )));
            };
            self.patch_a(jump, target);
        }
        Ok(())
    }

    pub(super) fn generic_for(
        &mut self,
        names: &[String],
        iter: &[Expr],
        body: &[Stmt],
    ) -> Result<()> {
        if names.is_empty() {
            return Err(FerretError::Unsupported(
                "generic for needs at least one variable".to_string(),
            ));
        }
        let iter_start = self.reserve(3);
        self.generic_iter_values(iter_start, iter)?;
        let name_regs = self.define_locals(names);
        let first_name = name_regs[0];
        let loop_start = self.instructions.len() as u16;
        self.breaks.push(Vec::new());
        self.emit(Op::GenericFor, first_name, iter_start, names.len() as u16);
        let exit = self.emit(Op::JmpFalse, first_name, 0, 0);
        self.stmts(body)?;
        self.emit(Op::Jmp, loop_start, 0, 0);
        let end = self.instructions.len() as u16;
        self.patch_b(exit, end);
        self.patch_breaks(end);
        Ok(())
    }

    pub(super) fn return_stmt(&mut self, values: &[Expr]) -> Result<()> {
        if matches!(values.last(), Some(Expr::VarArgs)) {
            return self.return_varargs(values);
        }
        let start = self.reserve(values.len() as u16);
        for (index, value) in values.iter().enumerate() {
            let src = self.expr(value)?;
            self.emit(Op::Move, start + index as u16, src, 0);
        }
        self.emit(Op::Return, start, values.len() as u16, 0);
        Ok(())
    }

    fn return_varargs(&mut self, values: &[Expr]) -> Result<()> {
        let fixed_count = values.len().saturating_sub(1);
        let start = self.reserve(fixed_count as u16);
        for (index, value) in values[..fixed_count].iter().enumerate() {
            let src = self.expr(value)?;
            self.emit(Op::Move, start + index as u16, src, 0);
        }
        self.emit(Op::ReturnVarArg, start, fixed_count as u16, 0);
        Ok(())
    }

    fn generic_iter_values(&mut self, dst: u16, iter: &[Expr]) -> Result<()> {
        if let [Expr::Call { callee, args }] = iter {
            return self.call3_into(callee, args, dst);
        }
        for index in 0..3 {
            let src = match iter.get(index) {
                Some(expr) => self.expr(expr)?,
                None => self.nil()?,
            };
            self.emit(Op::Move, dst + index as u16, src, 0);
        }
        Ok(())
    }
}
