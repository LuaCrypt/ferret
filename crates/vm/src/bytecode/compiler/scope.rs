use ferret_ir::Stmt;
use ferret_util::Result;

use super::{Binding, Compiler};

impl Compiler {
    pub(super) fn scoped_stmts(&mut self, stmts: &[Stmt]) -> Result<()> {
        let locals = self.locals.clone();
        let result = self.stmts(stmts);
        self.locals = locals;
        result
    }

    pub(super) fn restore_locals(&mut self, locals: std::collections::BTreeMap<String, Binding>) {
        self.locals = locals;
    }
}
