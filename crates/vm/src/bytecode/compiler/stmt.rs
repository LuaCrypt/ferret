use ferret_ir::{BinOp, Expr, Op, Stmt};
use ferret_util::{FerretError, Result};

use super::{Binding, Compiler};

enum FalseJump {
    B(usize),
    C(usize),
}

impl Compiler {
    pub(super) fn if_stmt(
        &mut self,
        cond: &Expr,
        then_body: &[Stmt],
        else_body: &[Stmt],
    ) -> Result<()> {
        if self.add_select_if(cond, then_body, else_body)? {
            return Ok(());
        }
        let false_jump = self.false_jump(cond)?;
        self.scoped_stmts(then_body)?;
        let end_jump = self.emit(Op::Jmp, 0, 0, 0);
        let else_start = self.instructions.len() as u16;
        self.patch_false_jump(false_jump, else_start);
        self.scoped_stmts(else_body)?;
        let end = self.instructions.len() as u16;
        self.patch_a(end_jump, end);
        Ok(())
    }

    pub(super) fn while_stmt(&mut self, cond: &Expr, body: &[Stmt]) -> Result<()> {
        let start = self.instructions.len() as u16;
        let end_jump = self.false_jump(cond)?;
        self.breaks.push(Vec::new());
        self.scoped_stmts(body)?;
        self.emit(Op::Jmp, start, 0, 0);
        let end = self.instructions.len() as u16;
        self.patch_false_jump(end_jump, end);
        self.patch_breaks(end);
        Ok(())
    }

    pub(super) fn repeat_stmt(&mut self, body: &[Stmt], cond: &Expr) -> Result<()> {
        let start = self.instructions.len() as u16;
        self.breaks.push(Vec::new());
        self.scoped_stmts(body)?;
        self.false_jump_to(cond, start)?;
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
        let start_reg = self.expr(start)?;
        let end_reg = self.expr(end)?;
        let step_reg = self.expr(step)?;
        let locals = self.locals.clone();
        let var = self.reserve(3);
        self.locals.insert(name.to_string(), Binding::Local(var));
        self.emit(Op::Move, var, start_reg, 0);
        self.emit(Op::Move, var + 1, end_reg, 0);
        self.emit(Op::Move, var + 2, step_reg, 0);
        let positive_step = matches!(step, Expr::Number(value) if *value >= 0.0);
        let check_op = if positive_step {
            Op::ForCheckPos
        } else {
            Op::ForCheck
        };
        let step_op = if positive_step {
            Op::ForStepPos
        } else {
            Op::ForStep
        };
        let exit = self.emit(check_op, var, 0, 0);
        let loop_start = self.instructions.len() as u16;
        if positive_step {
            if let Some(acc) = self.for_add_accumulator(name, body) {
                self.breaks.push(Vec::new());
                self.emit(Op::ForStepAddPos, acc, var, loop_start);
                let done = self.instructions.len() as u16;
                self.patch_b(exit, done);
                self.patch_breaks(done);
                self.restore_locals(locals);
                return Ok(());
            }
        }
        self.breaks.push(Vec::new());
        self.scoped_stmts(body)?;
        self.emit(step_op, var, loop_start, 0);
        let done = self.instructions.len() as u16;
        self.patch_b(exit, done);
        self.patch_breaks(done);
        self.restore_locals(locals);
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
        let locals = self.locals.clone();
        let iter_start = self.reserve(3);
        self.generic_iter_values(iter_start, iter)?;
        let name_regs = self.define_locals(names);
        let first_name = name_regs[0];
        let loop_start = self.instructions.len() as u16;
        self.breaks.push(Vec::new());
        let exit = if names.len() == 2 {
            self.emit(Op::GenericFor2Jmp, first_name, iter_start, 0)
        } else {
            self.emit(Op::GenericFor, first_name, iter_start, names.len() as u16);
            self.emit(Op::JmpFalse, first_name, 0, 0)
        };
        self.scoped_stmts(body)?;
        self.emit(Op::Jmp, loop_start, 0, 0);
        let end = self.instructions.len() as u16;
        if names.len() == 2 {
            self.patch_c(exit, end);
        } else {
            self.patch_b(exit, end);
        }
        self.patch_breaks(end);
        self.restore_locals(locals);
        Ok(())
    }

    pub(super) fn return_stmt(&mut self, values: &[Expr]) -> Result<()> {
        if matches!(values.last(), Some(Expr::VarArgs)) {
            return self.return_varargs(values);
        }
        if let Some(Expr::Call { callee, args }) = values.last() {
            return self.return_call(values, callee, args);
        }
        let start = self.reserve(values.len() as u16);
        for (index, value) in values.iter().enumerate() {
            let src = self.expr(value)?;
            self.emit(Op::Move, start + index as u16, src, 0);
        }
        self.emit(Op::Return, start, values.len() as u16, 0);
        Ok(())
    }

    fn return_call(&mut self, values: &[Expr], callee: &Expr, args: &[Expr]) -> Result<()> {
        let fixed_count = values.len().saturating_sub(1);
        let start = if fixed_count == 0 {
            self.next_reg
        } else {
            self.reserve(fixed_count as u16)
        };
        for (index, value) in values[..fixed_count].iter().enumerate() {
            let src = self.expr(value)?;
            self.emit(Op::Move, start + index as u16, src, 0);
        }
        let arg_start = self.open_call_args(callee, args)?;
        let counts = Self::packed_open_counts(fixed_count, args.len())?;
        self.emit(Op::ReturnCall, start, arg_start, counts);
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

    fn false_jump(&mut self, cond: &Expr) -> Result<FalseJump> {
        self.false_jump_to(cond, 0)
    }

    fn false_jump_to(&mut self, cond: &Expr, target: u16) -> Result<FalseJump> {
        if let Expr::Binary { op, left, right } = cond {
            if let Some((jump_op, left, right)) = comparison_false_jump(*op, left, right) {
                let left = self.expr(left)?;
                let right = self.expr(right)?;
                return Ok(FalseJump::C(self.emit(jump_op, left, right, target)));
            }
        }
        let cond = self.expr(cond)?;
        Ok(FalseJump::B(self.emit(Op::JmpFalse, cond, target, 0)))
    }

    fn patch_false_jump(&mut self, jump: FalseJump, target: u16) {
        match jump {
            FalseJump::B(index) => self.patch_b(index, target),
            FalseJump::C(index) => self.patch_c(index, target),
        }
    }
}

fn comparison_false_jump<'a>(
    op: BinOp,
    left: &'a Expr,
    right: &'a Expr,
) -> Option<(Op, &'a Expr, &'a Expr)> {
    Some(match op {
        BinOp::Eq => (Op::JmpNotEq, left, right),
        BinOp::Lt => (Op::JmpNotLt, left, right),
        BinOp::Le => (Op::JmpNotLe, left, right),
        BinOp::Gt => (Op::JmpNotLt, right, left),
        BinOp::Ge => (Op::JmpNotLe, right, left),
        _ => return None,
    })
}
