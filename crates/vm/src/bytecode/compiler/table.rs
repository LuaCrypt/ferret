use ferret_ir::{Const, Expr, Op};
use ferret_util::{FerretError, Result};

use super::Compiler;

impl Compiler {
    pub(super) fn table(&mut self, fields: &[(Option<Expr>, Expr)]) -> Result<u16> {
        let dst = self.alloc();
        self.emit(Op::NewTable, dst, 0, 0);
        let mut array_index = 1;
        for (field_index, (key, value)) in fields.iter().enumerate() {
            if key.is_none() && field_index + 1 == fields.len() {
                if let Expr::Call { callee, args } = value {
                    if array_index > u8::MAX as usize {
                        return Err(FerretError::Unsupported(
                            "open table multireturn index is too large for the VM subset"
                                .to_string(),
                        ));
                    }
                    let arg_start = self.open_call_args(callee, args)?;
                    let counts = Self::packed_open_counts(array_index, args.len())?;
                    self.emit(Op::SetTableCall, dst, arg_start, counts);
                    continue;
                }
                if matches!(value, Expr::VarArgs) {
                    self.emit(Op::SetTableVarArg, dst, array_index as u16, 0);
                    continue;
                }
            }
            let key = match key {
                Some(key) => self.expr(key)?,
                None => {
                    let reg = self.load_const(Const::Number(array_index.to_string()))?;
                    array_index += 1;
                    reg
                }
            };
            let value = self.expr(value)?;
            self.emit(Op::SetTable, dst, key, value);
        }
        Ok(dst)
    }
}
