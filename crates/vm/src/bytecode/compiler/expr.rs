use ferret_ir::{BinOp, Const, Expr, Op, UnOp};
use ferret_util::{FerretError, Result};

use super::{Binding, Compiler, EnvBinding};
use crate::bytecode::support::bin_op;

impl Compiler {
    pub(super) fn expr(&mut self, expr: &Expr) -> Result<u16> {
        match expr {
            Expr::Nil => self.load_const(Const::Nil),
            Expr::Bool(value) => self.load_const(Const::Bool(*value)),
            Expr::Number(value) => self.load_const(Const::Number(*value)),
            Expr::String(value) => self.load_const(Const::String(value.clone())),
            Expr::VarArgs => Err(FerretError::Unsupported(
                "varargs are only supported in return tails".to_string(),
            )),
            Expr::Var(name) => self.var(name),
            Expr::Table(fields) => self.table(fields),
            Expr::Unary { op, expr } => self.unary(*op, expr),
            Expr::Binary { op, left, right } => self.binary(*op, left, right),
            Expr::Call { callee, args } => self.call(callee, args),
            Expr::Function {
                params,
                vararg: _,
                body,
            } => self.function(params, body),
            Expr::Index { table, key } => {
                let dst = self.alloc();
                let table = self.expr(table)?;
                let key = self.expr(key)?;
                self.emit(Op::GetTable, dst, table, key);
                Ok(dst)
            }
        }
    }

    pub(super) fn define_locals(&mut self, names: &[String]) -> Vec<u16> {
        names
            .iter()
            .map(|name| {
                let dst = self.alloc();
                self.locals.insert(name.clone(), Binding::Local(dst));
                dst
            })
            .collect()
    }

    pub(super) fn bind_local_regs(&mut self, names: &[String], regs: &[u16]) {
        for (name, reg) in names.iter().zip(regs.iter().copied()) {
            self.locals.insert(name.clone(), Binding::Local(reg));
        }
    }

    pub(super) fn value_regs(&mut self, values: &[Expr], target_count: usize) -> Result<Vec<u16>> {
        if let Some(Expr::Call { callee, args }) = values.last() {
            let fixed_count = values.len() - 1;
            if target_count > fixed_count + 1 {
                let mut regs = self.value_regs(&values[..fixed_count], fixed_count)?;
                let call_count = target_count - fixed_count;
                let dst = self.next_reg;
                self.call_n_into(callee, args, dst, call_count as u16)?;
                regs.extend((0..call_count).map(|index| dst + index as u16));
                return Ok(regs);
            }
        }
        values.iter().map(|value| self.expr(value)).collect()
    }

    pub(super) fn assign_target(&mut self, target: &Expr, src: u16) -> Result<()> {
        match target {
            Expr::Var(name) => self.assign_var(name, src)?,
            Expr::Index { table, key } => {
                let table = self.expr(table)?;
                let key = self.expr(key)?;
                self.emit(Op::SetTable, table, key, src);
            }
            _ => {
                return Err(FerretError::Unsupported(
                    "assignment target is not in the VM subset yet".to_string(),
                ));
            }
        }
        Ok(())
    }

    pub(super) fn expr_into(&mut self, dst: u16, expr: &Expr) -> Result<()> {
        match expr {
            Expr::Nil => self.load_const_into(dst, Const::Nil)?,
            Expr::Bool(value) => self.load_const_into(dst, Const::Bool(*value))?,
            Expr::Number(value) => self.load_const_into(dst, Const::Number(*value))?,
            Expr::String(value) => self.load_const_into(dst, Const::String(value.clone()))?,
            Expr::Unary { op, expr } => {
                let src = self.expr(expr)?;
                let op = match op {
                    UnOp::Neg => Op::Neg,
                    UnOp::Not => Op::Not,
                    UnOp::Len => Op::Len,
                    UnOp::BitNot => Op::BitNot,
                };
                self.emit(op, dst, src, 0);
            }
            Expr::Binary { op, left, right } => self.binary_into(dst, *op, left, right)?,
            Expr::Var(name) => {
                let src = self.var(name)?;
                if src != dst {
                    self.emit(Op::Move, dst, src, 0);
                }
            }
            _ => {
                let src = self.expr(expr)?;
                if src != dst {
                    self.emit(Op::Move, dst, src, 0);
                }
            }
        }
        Ok(())
    }

    pub(super) fn nil(&mut self) -> Result<u16> {
        self.load_const(Const::Nil)
    }

    fn load_const_into(&mut self, dst: u16, value: Const) -> Result<()> {
        let key = self.constant(value)?;
        self.emit(Op::LoadK, dst, key, 0);
        Ok(())
    }

    fn var(&mut self, name: &str) -> Result<u16> {
        if let Some(binding) = self.locals.get(name).copied() {
            self.read_binding(binding)
        } else if let Some(upvalue) = self.upvalues.get(name).copied() {
            let dst = self.alloc();
            self.emit(Op::GetUp, dst, upvalue, 0);
            Ok(dst)
        } else if let Some(env) = self.env_binding() {
            let dst = self.alloc();
            let table = self.read_env_binding(env)?;
            let key = self.load_const(Const::String(name.to_string()))?;
            self.emit(Op::GetTable, dst, table, key);
            Ok(dst)
        } else {
            let dst = self.alloc();
            let key = self.constant(Const::String(name.to_string()))?;
            self.emit(Op::GetGlobal, dst, key, 0);
            Ok(dst)
        }
    }

    fn assign_var(&mut self, name: &str, src: u16) -> Result<()> {
        if let Some(binding) = self.locals.get(name).copied() {
            self.write_binding(binding, src);
        } else if let Some(upvalue) = self.upvalues.get(name).copied() {
            self.emit(Op::SetUp, upvalue, src, 0);
        } else if let Some(env) = self.env_binding() {
            let table = self.read_env_binding(env)?;
            let key = self.load_const(Const::String(name.to_string()))?;
            self.emit(Op::SetTable, table, key, src);
        } else {
            let key = self.constant(Const::String(name.to_string()))?;
            self.emit(Op::SetGlobal, key, src, 0);
        }
        Ok(())
    }

    fn unary(&mut self, op: UnOp, expr: &Expr) -> Result<u16> {
        let dst = self.alloc();
        let src = self.expr(expr)?;
        let op = match op {
            UnOp::Neg => Op::Neg,
            UnOp::Not => Op::Not,
            UnOp::Len => Op::Len,
            UnOp::BitNot => Op::BitNot,
        };
        self.emit(op, dst, src, 0);
        Ok(dst)
    }

    fn binary(&mut self, op: BinOp, left: &Expr, right: &Expr) -> Result<u16> {
        let dst = self.alloc();
        self.binary_into(dst, op, left, right)?;
        Ok(dst)
    }

    fn binary_into(&mut self, dst: u16, op: BinOp, left: &Expr, right: &Expr) -> Result<()> {
        match op {
            BinOp::Ne => {
                let tmp = self.binary(BinOp::Eq, left, right)?;
                self.emit(Op::Not, dst, tmp, 0);
            }
            BinOp::Gt => self.reverse_compare(Op::Lt, dst, left, right)?,
            BinOp::Ge => self.reverse_compare(Op::Le, dst, left, right)?,
            _ if const_bin_op(op).is_some() && scalar_const(right).is_some() => {
                let left = self.expr(left)?;
                let key = self.constant(scalar_const(right).expect("checked above"))?;
                self.emit(const_bin_op(op).expect("checked above"), dst, left, key);
            }
            _ => {
                let left = self.expr(left)?;
                let right = self.expr(right)?;
                self.emit(bin_op(op)?, dst, left, right);
            }
        }
        Ok(())
    }

    fn reverse_compare(&mut self, op: Op, dst: u16, left: &Expr, right: &Expr) -> Result<()> {
        let left = self.expr(left)?;
        let right = self.expr(right)?;
        self.emit(op, dst, right, left);
        Ok(())
    }

    pub(super) fn env_binding(&self) -> Option<EnvBinding> {
        if let Some(binding) = self.locals.get("_ENV").copied() {
            Some(EnvBinding::Local(binding))
        } else {
            self.upvalues.get("_ENV").copied().map(EnvBinding::Upvalue)
        }
    }

    fn read_env_binding(&mut self, binding: EnvBinding) -> Result<u16> {
        match binding {
            EnvBinding::Local(binding) => self.read_binding(binding),
            EnvBinding::Upvalue(upvalue) => {
                let dst = self.alloc();
                self.emit(Op::GetUp, dst, upvalue, 0);
                Ok(dst)
            }
        }
    }

    fn read_binding(&mut self, binding: Binding) -> Result<u16> {
        match binding {
            Binding::Local(reg) => Ok(reg),
            Binding::Cell(reg) => {
                let dst = self.alloc();
                self.emit(Op::GetCell, dst, reg, 0);
                Ok(dst)
            }
        }
    }

    pub(super) fn write_binding(&mut self, binding: Binding, src: u16) {
        match binding {
            Binding::Local(dst) => {
                self.emit(Op::Move, dst, src, 0);
            }
            Binding::Cell(dst) => {
                self.emit(Op::SetCell, dst, src, 0);
            }
        }
    }
}

fn const_bin_op(op: BinOp) -> Option<Op> {
    Some(match op {
        BinOp::Add => Op::AddK,
        BinOp::Sub => Op::SubK,
        BinOp::Mul => Op::MulK,
        BinOp::Div => Op::DivK,
        BinOp::FloorDiv => Op::FloorDivK,
        BinOp::Mod => Op::ModK,
        BinOp::Pow => Op::PowK,
        _ => return None,
    })
}

fn scalar_const(expr: &Expr) -> Option<Const> {
    match expr {
        Expr::Number(value) => Some(Const::Number(*value)),
        Expr::String(value) => Some(Const::String(value.clone())),
        _ => None,
    }
}
