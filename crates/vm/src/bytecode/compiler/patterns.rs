use ferret_ir::{BinOp, Const, Expr, Op, Stmt};
use ferret_util::Result;

use super::{Binding, Compiler};

impl Compiler {
    pub(super) fn special_assign(&mut self, targets: &[Expr], values: &[Expr]) -> Result<bool> {
        let ([target], [value]) = (targets, values) else {
            return Ok(false);
        };
        let Some(dst) = self.local_target(target) else {
            return Ok(false);
        };
        if self.add_mod_assign(dst, value)? {
            return Ok(true);
        }
        if self.mul_add_mod_assign(dst, value)? {
            return Ok(true);
        }
        if self.add_assign(dst, value)? {
            return Ok(true);
        }
        Ok(false)
    }

    pub(super) fn add_select_if(
        &mut self,
        cond: &Expr,
        then_body: &[Stmt],
        else_body: &[Stmt],
    ) -> Result<bool> {
        let Expr::Binary {
            op: BinOp::Lt,
            left,
            right,
        } = cond
        else {
            return Ok(false);
        };
        let Some(left_reg) = self.local_source(left) else {
            return Ok(false);
        };
        let Some(right_reg) = self.local_source(right) else {
            return Ok(false);
        };
        let Some((then_dst, then_src)) = add_assign(then_body) else {
            return Ok(false);
        };
        let Some((else_dst, else_src)) = add_assign(else_body) else {
            return Ok(false);
        };
        if then_dst != else_dst
            || Some(then_src) != var_name(left)
            || Some(else_src) != var_name(right)
        {
            return Ok(false);
        }
        let Some(acc) = self.local_name(then_dst) else {
            return Ok(false);
        };
        self.emit(Op::AddSelectLt, acc, left_reg, right_reg);
        Ok(true)
    }

    pub(super) fn for_add_accumulator(&self, loop_name: &str, body: &[Stmt]) -> Option<u16> {
        let [Stmt::Assign { targets, values }] = body else {
            return None;
        };
        let ([target], [value]) = (&targets[..], &values[..]) else {
            return None;
        };
        let target = var_name(target)?;
        let Expr::Binary {
            op: BinOp::Add,
            left,
            right,
        } = value
        else {
            return None;
        };
        match (var_name(left), var_name(right)) {
            (Some(left), Some(right)) if left == target && right == loop_name => {
                self.local_name(target)
            }
            (Some(left), Some(right)) if right == target && left == loop_name => {
                self.local_name(target)
            }
            _ => None,
        }
    }

    fn add_mod_assign(&mut self, dst: u16, value: &Expr) -> Result<bool> {
        let Expr::Binary {
            op: BinOp::Mod,
            left,
            right,
        } = value
        else {
            return Ok(false);
        };
        let Some(mod_key) = self.scalar_key(right)? else {
            return Ok(false);
        };
        let Expr::Binary {
            op: BinOp::Add,
            left,
            right,
        } = left.as_ref()
        else {
            return Ok(false);
        };
        let Some(rhs) = self.same_dest_plus_rhs(dst, left, right) else {
            return Ok(false);
        };
        self.emit(Op::AddModK, dst, rhs, mod_key);
        Ok(true)
    }

    fn mul_add_mod_assign(&mut self, dst: u16, value: &Expr) -> Result<bool> {
        let Expr::Binary {
            op: BinOp::Mod,
            left,
            right,
        } = value
        else {
            return Ok(false);
        };
        let Some(mod_key) = self.scalar_key(right)? else {
            return Ok(false);
        };
        let Expr::Binary {
            op: BinOp::Add,
            left,
            right,
        } = left.as_ref()
        else {
            return Ok(false);
        };
        let Some((mul_key, add_src)) = self.mul_dest_plus_rhs(dst, left, right)? else {
            return Ok(false);
        };
        if mul_key > 255 || mod_key > 255 {
            return Ok(false);
        }
        self.emit(Op::MulKAddModK, dst, add_src, (mul_key << 8) | mod_key);
        Ok(true)
    }

    fn add_assign(&mut self, dst: u16, value: &Expr) -> Result<bool> {
        let Expr::Binary {
            op: BinOp::Add,
            left,
            right,
        } = value
        else {
            return Ok(false);
        };
        if let Some(key) = self.same_dest_plus_key(dst, left, right)? {
            self.emit(Op::AddK, dst, dst, key);
            return Ok(true);
        }
        if let Some(rhs) = self.same_dest_plus_rhs(dst, left, right) {
            self.emit(Op::Add, dst, dst, rhs);
            return Ok(true);
        }
        Ok(false)
    }

    fn same_dest_plus_rhs(&self, dst: u16, left: &Expr, right: &Expr) -> Option<u16> {
        match (self.local_source(left), self.local_source(right)) {
            (Some(left), Some(right)) if left == dst => Some(right),
            (Some(left), Some(right)) if right == dst => Some(left),
            _ => None,
        }
    }

    fn same_dest_plus_key(&mut self, dst: u16, left: &Expr, right: &Expr) -> Result<Option<u16>> {
        match (self.local_source(left), self.scalar_key(right)?) {
            (Some(left), Some(key)) if left == dst => Ok(Some(key)),
            _ => match (self.local_source(right), self.scalar_key(left)?) {
                (Some(right), Some(key)) if right == dst => Ok(Some(key)),
                _ => Ok(None),
            },
        }
    }

    fn mul_dest_plus_rhs(
        &mut self,
        dst: u16,
        left: &Expr,
        right: &Expr,
    ) -> Result<Option<(u16, u16)>> {
        if let Some(mul_key) = self.mul_dest_key(dst, left)? {
            return Ok(self.local_source(right).map(|src| (mul_key, src)));
        }
        if let Some(mul_key) = self.mul_dest_key(dst, right)? {
            return Ok(self.local_source(left).map(|src| (mul_key, src)));
        }
        Ok(None)
    }

    fn mul_dest_key(&mut self, dst: u16, expr: &Expr) -> Result<Option<u16>> {
        let Expr::Binary {
            op: BinOp::Mul,
            left,
            right,
        } = expr
        else {
            return Ok(None);
        };
        match (self.local_source(left), self.scalar_key(right)?) {
            (Some(src), Some(key)) if src == dst => Ok(Some(key)),
            _ => match (self.local_source(right), self.scalar_key(left)?) {
                (Some(src), Some(key)) if src == dst => Ok(Some(key)),
                _ => Ok(None),
            },
        }
    }

    fn scalar_key(&mut self, expr: &Expr) -> Result<Option<u16>> {
        Ok(match expr {
            Expr::Number(value) => Some(self.constant(Const::Number(value.clone()))?),
            Expr::String(value) => Some(self.constant(Const::String(value.clone()))?),
            _ => None,
        })
    }

    fn local_target(&self, expr: &Expr) -> Option<u16> {
        var_name(expr).and_then(|name| self.local_name(name))
    }

    fn local_source(&self, expr: &Expr) -> Option<u16> {
        var_name(expr).and_then(|name| self.local_name(name))
    }

    fn local_name(&self, name: &str) -> Option<u16> {
        match self.locals.get(name).copied() {
            Some(Binding::Local(reg)) => Some(reg),
            _ => None,
        }
    }
}

fn add_assign(stmts: &[Stmt]) -> Option<(&str, &str)> {
    let [Stmt::Assign { targets, values }] = stmts else {
        return None;
    };
    let ([target], [value]) = (&targets[..], &values[..]) else {
        return None;
    };
    let target = var_name(target)?;
    let Expr::Binary {
        op: BinOp::Add,
        left,
        right,
    } = value
    else {
        return None;
    };
    match (var_name(left), var_name(right)) {
        (Some(left), Some(right)) if left == target => Some((target, right)),
        (Some(left), Some(right)) if right == target => Some((target, left)),
        _ => None,
    }
}

fn var_name(expr: &Expr) -> Option<&str> {
    match expr {
        Expr::Var(name) => Some(name),
        _ => None,
    }
}
