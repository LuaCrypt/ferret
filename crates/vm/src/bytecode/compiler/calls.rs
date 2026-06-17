use std::collections::BTreeMap;

use ferret_ir::{Capture, Chunk, Const, Expr, Op, Stmt};
use ferret_util::{FerretError, Result};

use super::{captures::collect_captures, Binding, Compiler};

impl Compiler {
    pub(super) fn call(&mut self, callee: &Expr, args: &[Expr]) -> Result<u16> {
        let dst = self.alloc();
        let func = self.expr(callee)?;
        let arg_start = self.next_reg;
        self.reserve(args.len() as u16);
        self.move_args(arg_start, args)?;
        self.emit(Op::Call, dst, func, arg_start);
        self.emit_arg_count(args.len() as u16);
        Ok(dst)
    }

    pub(super) fn call3_into(&mut self, callee: &Expr, args: &[Expr], dst: u16) -> Result<()> {
        let func = self.expr(callee)?;
        let arg_start = self.next_reg;
        self.reserve(args.len() as u16);
        self.move_args(arg_start, args)?;
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
        let func = self.expr(callee)?;
        let arg_start = dst + count;
        self.reserve_to(arg_start + args.len() as u16);
        self.move_args(arg_start, args)?;
        self.emit(Op::CallN, dst, func, (count << 8) | args.len() as u16);
        Ok(())
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

    fn move_args(&mut self, start: u16, args: &[Expr]) -> Result<()> {
        for (index, arg) in args.iter().enumerate() {
            let src = self.expr(arg)?;
            self.emit(Op::Move, start + index as u16, src, 0);
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
