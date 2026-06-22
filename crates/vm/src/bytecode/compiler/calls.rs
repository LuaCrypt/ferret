use std::collections::BTreeMap;

use ferret_ir::{Capture, Chunk, Const, Expr, Op, Stmt};
use ferret_util::{FerretError, Result};

use super::{captures::collect_captures, Binding, Compiler};

impl Compiler {
    pub(super) fn call(&mut self, callee: &Expr, args: &[Expr]) -> Result<u16> {
        if let Some(fixed) = fixed_args_before_varargs(args) {
            return self.call_varargs(callee, fixed);
        }
        if let Some((fixed, tail_callee, tail_args)) = fixed_args_before_open_call(args) {
            return self.call_open(callee, fixed, tail_callee, tail_args);
        }
        let dst = self.alloc();
        let (func, arg_start) = self.call_parts(callee, args)?;
        self.emit(Op::Call, dst, func, arg_start);
        self.emit_arg_count(args.len() as u16);
        Ok(dst)
    }

    pub(super) fn call_global_stmt(&mut self, callee: &Expr, args: &[Expr]) -> Result<bool> {
        let Expr::Var(name) = callee else {
            return Ok(false);
        };
        if fixed_args_before_varargs(args).is_some() {
            return Ok(false);
        }
        if fixed_args_before_open_call(args).is_some() {
            return Ok(false);
        }
        if self.locals.contains_key(name)
            || self.upvalues.contains_key(name)
            || self.env_binding().is_some()
        {
            return Ok(false);
        }
        let key = self.constant(Const::String(name.clone()))?;
        let arg_start = self.args_after_current(args)?;
        self.emit(Op::CallGlobal, key, arg_start, args.len() as u16);
        Ok(true)
    }

    pub(super) fn call3_into(&mut self, callee: &Expr, args: &[Expr], dst: u16) -> Result<()> {
        let (func, arg_start) = self.call_parts(callee, args)?;
        self.emit(Op::Call3, dst, func, arg_start);
        self.emit_arg_count(args.len() as u16);
        Ok(())
    }

    pub(super) fn call_n_into(
        &mut self,
        callee: &Expr,
        args: &[Expr],
        dst: u16,
        count: u16,
    ) -> Result<()> {
        if let Some(fixed) = fixed_args_before_varargs(args) {
            return self.call_n_varargs_into(callee, fixed, dst, count);
        }
        if let Some((fixed, tail_callee, tail_args)) = fixed_args_before_open_call(args) {
            return self.call_n_open_into(callee, fixed, tail_callee, tail_args, dst, count);
        }
        let arg_start = dst + count;
        let mut func = self.expr(callee)?;
        if func >= arg_start && func < arg_start + args.len() as u16 {
            let slot = self.next_reg;
            self.reserve(1);
            self.emit(Op::Move, slot, func, 0);
            func = slot;
        }
        self.args_at(arg_start, args)?;
        self.emit(Op::CallN, dst, func, (count << 8) | args.len() as u16);
        Ok(())
    }

    pub(super) fn open_call_args(&mut self, callee: &Expr, args: &[Expr]) -> Result<u16> {
        if args.len() > u8::MAX as usize {
            return Err(FerretError::Unsupported(
                "calls with more than 255 arguments are not in the VM subset yet".to_string(),
            ));
        }
        let func = self.expr(callee)?;
        let func_slot = self.next_reg;
        self.reserve(1);
        self.emit(Op::Move, func_slot, func, 0);
        let arg_start = func_slot + 1;
        self.args_at(arg_start, args)?;
        Ok(arg_start)
    }

    pub(super) fn packed_open_counts(fixed_count: usize, arg_count: usize) -> Result<u16> {
        if fixed_count > u8::MAX as usize || arg_count > u8::MAX as usize {
            return Err(FerretError::Unsupported(
                "open multireturn call shape is too large for the VM subset".to_string(),
            ));
        }
        Ok(((fixed_count as u16) << 8) | arg_count as u16)
    }

    pub(super) fn function(&mut self, params: &[String], body: &[Stmt]) -> Result<u16> {
        let captures = collect_captures(params, body, &self.capture_names());
        let mut upvalues = BTreeMap::new();
        let mut runtime_captures = Vec::with_capacity(captures.len());
        for (index, name) in captures.iter().enumerate() {
            upvalues.insert(name.clone(), index as u16);
            runtime_captures.push(self.capture(name)?);
        }
        let mut child = Compiler::with_params(params, upvalues);
        child.stmts(body)?;
        child.finish_gotos()?;
        child.emit(Op::Halt, 0, 0, 0);
        self.load_const(Const::Function {
            chunk: Box::new(Chunk {
                constants: child.constants,
                instructions: child.instructions,
                registers: child.max_reg,
                params: params.len() as u16,
            }),
            captures: runtime_captures,
        })
    }

    pub(super) fn load_const(&mut self, value: Const) -> Result<u16> {
        let dst = self.alloc();
        let key = self.constant(value)?;
        self.emit(Op::LoadK, dst, key, 0);
        Ok(dst)
    }

    fn call_parts(&mut self, callee: &Expr, args: &[Expr]) -> Result<(u16, u16)> {
        let func = self.expr(callee)?;
        let arg_start = self.args_after_current(args)?;
        Ok((func, arg_start))
    }

    fn call_varargs(&mut self, callee: &Expr, args: &[Expr]) -> Result<u16> {
        let dst = self.alloc();
        let func = self.expr(callee)?;
        let arg_start = self.args_after_current(args)?;
        self.emit(
            Op::CallVarArg,
            dst,
            func,
            packed_start_count(arg_start, args.len())?,
        );
        Ok(dst)
    }

    fn call_open(
        &mut self,
        callee: &Expr,
        fixed: &[Expr],
        tail_callee: &Expr,
        tail_args: &[Expr],
    ) -> Result<u16> {
        let dst = self.alloc();
        let func = self.open_func_slot(callee)?;
        self.open_fixed_and_tail(func + 1, fixed, tail_callee, tail_args)?;
        let counts = Self::packed_open_counts(fixed.len(), tail_args.len())?;
        self.emit(Op::CallOpen, dst, func, counts);
        Ok(dst)
    }

    fn call_n_varargs_into(
        &mut self,
        callee: &Expr,
        args: &[Expr],
        dst: u16,
        count: u16,
    ) -> Result<()> {
        let arg_start = dst + count;
        let mut func = self.expr(callee)?;
        if func >= arg_start && func < arg_start + args.len() as u16 {
            let slot = self.next_reg;
            self.reserve(1);
            self.emit(Op::Move, slot, func, 0);
            func = slot;
        }
        self.args_at(arg_start, args)?;
        self.emit(Op::CallNVarArg, dst, func, (count << 8) | args.len() as u16);
        Ok(())
    }

    fn call_n_open_into(
        &mut self,
        callee: &Expr,
        fixed: &[Expr],
        tail_callee: &Expr,
        tail_args: &[Expr],
        dst: u16,
        count: u16,
    ) -> Result<()> {
        let func = dst + count;
        self.reserve_to(func + 1);
        let src = self.expr(callee)?;
        self.emit(Op::Move, func, src, 0);
        self.open_fixed_and_tail(func + 1, fixed, tail_callee, tail_args)?;
        let counts = Self::packed_open_counts(fixed.len(), tail_args.len())?;
        self.emit(Op::CallNOpen, dst, func, counts);
        Ok(())
    }

    fn open_func_slot(&mut self, callee: &Expr) -> Result<u16> {
        let slot = self.next_reg;
        self.reserve(1);
        let src = self.expr(callee)?;
        self.emit(Op::Move, slot, src, 0);
        Ok(slot)
    }

    pub(super) fn open_fixed_and_tail(
        &mut self,
        start: u16,
        fixed: &[Expr],
        tail_callee: &Expr,
        tail_args: &[Expr],
    ) -> Result<()> {
        self.args_at(start, fixed)?;
        let tail_func = start + fixed.len() as u16;
        self.reserve_to(tail_func + 1);
        let src = self.expr(tail_callee)?;
        self.emit(Op::Move, tail_func, src, 0);
        self.args_at(tail_func + 1, tail_args)?;
        Ok(())
    }

    fn args_after_current(&mut self, args: &[Expr]) -> Result<u16> {
        let arg_start = self.next_reg;
        self.args_at(arg_start, args)
    }

    fn args_at(&mut self, arg_start: u16, args: &[Expr]) -> Result<u16> {
        self.reserve_to(arg_start + args.len() as u16);
        self.move_args(arg_start, args)?;
        Ok(arg_start)
    }

    pub(super) fn move_args(&mut self, start: u16, args: &[Expr]) -> Result<()> {
        for (index, arg) in args.iter().enumerate() {
            self.expr_into(start + index as u16, arg)?;
        }
        Ok(())
    }

    fn emit_arg_count(&mut self, count: u16) {
        let last = self.instructions.last_mut().expect("call already emitted");
        last.c = (last.c << 8) | count;
    }

    fn capture(&mut self, name: &str) -> Result<Capture> {
        if self.locals.contains_key(name) {
            Ok(Capture::Local(self.ensure_cell(name)?))
        } else if let Some(upvalue) = self.upvalues.get(name).copied() {
            Ok(Capture::Upvalue(upvalue))
        } else {
            Err(FerretError::Compile("missing closure capture".to_string()))
        }
    }

    fn ensure_cell(&mut self, name: &str) -> Result<u16> {
        match self.locals.get(name).copied() {
            Some(Binding::Cell(reg)) => Ok(reg),
            Some(Binding::Local(reg)) => {
                self.emit(Op::Cell, reg, reg, 0);
                self.locals.insert(name.to_string(), Binding::Cell(reg));
                Ok(reg)
            }
            None => Err(FerretError::Compile("missing local binding".to_string())),
        }
    }
}

fn fixed_args_before_varargs(args: &[Expr]) -> Option<&[Expr]> {
    let (last, fixed) = args.split_last()?;
    matches!(last, Expr::VarArgs).then_some(fixed)
}

fn fixed_args_before_open_call(args: &[Expr]) -> Option<(&[Expr], &Expr, &[Expr])> {
    let (last, fixed) = args.split_last()?;
    match last {
        Expr::Call { callee, args } => Some((fixed, callee, args)),
        _ => None,
    }
}

fn packed_start_count(start: u16, count: usize) -> Result<u16> {
    if start > u8::MAX as u16 || count > u8::MAX as usize {
        return Err(FerretError::Unsupported(
            "vararg call shape is too large for the VM subset".to_string(),
        ));
    }
    Ok((start << 8) | count as u16)
}
