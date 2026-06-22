use ferret_ir::{Expr, Program, Stmt};
use ferret_util::{FerretError, Result};

pub(super) fn reject_hostile(program: &Program, allow_dynamic_loaders: bool) -> Result<()> {
    reject_hostile_stmts(&program.body, allow_dynamic_loaders)
}

fn reject_hostile_stmts(stmts: &[Stmt], allow_dynamic_loaders: bool) -> Result<()> {
    for stmt in stmts {
        reject_hostile_stmt(stmt, allow_dynamic_loaders)?;
    }
    Ok(())
}

fn reject_hostile_stmt(stmt: &Stmt, allow_dynamic_loaders: bool) -> Result<()> {
    match stmt {
        Stmt::Local { values, .. } | Stmt::Return(values) => {
            reject_hostile_exprs(values, allow_dynamic_loaders)?;
        }
        Stmt::Assign { targets, values } => {
            reject_hostile_exprs(targets, allow_dynamic_loaders)?;
            reject_hostile_exprs(values, allow_dynamic_loaders)?;
        }
        Stmt::Block(body) => reject_hostile_stmts(body, allow_dynamic_loaders)?,
        Stmt::Expr(expr) => reject_hostile_expr(expr, allow_dynamic_loaders)?,
        Stmt::If {
            cond,
            then_body,
            else_body,
        } => {
            reject_hostile_expr(cond, allow_dynamic_loaders)?;
            reject_hostile_stmts(then_body, allow_dynamic_loaders)?;
            reject_hostile_stmts(else_body, allow_dynamic_loaders)?;
        }
        Stmt::While { cond, body } | Stmt::Repeat { cond, body } => {
            reject_hostile_expr(cond, allow_dynamic_loaders)?;
            reject_hostile_stmts(body, allow_dynamic_loaders)?;
        }
        Stmt::NumericFor {
            start,
            end,
            step,
            body,
            ..
        } => {
            reject_hostile_expr(start, allow_dynamic_loaders)?;
            reject_hostile_expr(end, allow_dynamic_loaders)?;
            reject_hostile_expr(step, allow_dynamic_loaders)?;
            reject_hostile_stmts(body, allow_dynamic_loaders)?;
        }
        Stmt::GenericFor { iter, body, .. } => {
            reject_hostile_exprs(iter, allow_dynamic_loaders)?;
            reject_hostile_stmts(body, allow_dynamic_loaders)?;
        }
        Stmt::Break | Stmt::Label(_) | Stmt::Goto(_) => {}
    }
    Ok(())
}

fn reject_hostile_exprs(exprs: &[Expr], allow_dynamic_loaders: bool) -> Result<()> {
    for expr in exprs {
        reject_hostile_expr(expr, allow_dynamic_loaders)?;
    }
    Ok(())
}

fn reject_hostile_expr(expr: &Expr, allow_dynamic_loaders: bool) -> Result<()> {
    reject_hostile_reference(expr, allow_dynamic_loaders)?;
    match expr {
        Expr::Table(fields) => {
            for (key, value) in fields {
                if let Some(key) = key {
                    reject_hostile_expr(key, allow_dynamic_loaders)?;
                }
                reject_hostile_expr(value, allow_dynamic_loaders)?;
            }
        }
        Expr::Unary { expr, .. } => reject_hostile_expr(expr, allow_dynamic_loaders)?,
        Expr::Binary { left, right, .. } => {
            reject_hostile_expr(left, allow_dynamic_loaders)?;
            reject_hostile_expr(right, allow_dynamic_loaders)?;
        }
        Expr::Call { callee, args } => {
            reject_hostile_call(callee, args)?;
            reject_hostile_expr(callee, allow_dynamic_loaders)?;
            reject_hostile_exprs(args, allow_dynamic_loaders)?;
        }
        Expr::Function { body, .. } => reject_hostile_stmts(body, allow_dynamic_loaders)?,
        Expr::Index { table, key } => {
            reject_hostile_expr(table, allow_dynamic_loaders)?;
            reject_hostile_expr(key, allow_dynamic_loaders)?;
        }
        Expr::Nil
        | Expr::Bool(_)
        | Expr::Number(_)
        | Expr::String(_)
        | Expr::VarArgs
        | Expr::Var(_) => {}
    }
    Ok(())
}

fn reject_hostile_reference(expr: &Expr, allow_dynamic_loaders: bool) -> Result<()> {
    if let Some(name) = hostile_reference_name(expr, allow_dynamic_loaders) {
        return Err(FerretError::Unsupported(format!(
            "'{name}' is rejected by the VM-only profile"
        )));
    }
    Ok(())
}

fn reject_hostile_call(callee: &Expr, args: &[Expr]) -> Result<()> {
    if matches!(callee, Expr::Var(name) if name == "require")
        && matches!(args.first(), Some(Expr::String(module)) if module == "debug")
    {
        return Err(FerretError::Unsupported(
            "'debug' is rejected by the VM-only profile".to_string(),
        ));
    }
    Ok(())
}

fn hostile_reference_name(expr: &Expr, allow_dynamic_loaders: bool) -> Option<&'static str> {
    match expr {
        Expr::Var(name) => hostile_global(name, allow_dynamic_loaders),
        Expr::Index { table, key } => match (&**table, &**key) {
            (Expr::Var(namespace), Expr::String(method))
                if namespace == "debug" && !method.is_empty() =>
            {
                Some("debug")
            }
            (Expr::Var(namespace), Expr::String(method))
                if namespace == "coroutine" && method == "yield" =>
            {
                Some("coroutine.yield")
            }
            (Expr::Var(namespace), Expr::String(name))
                if namespace == "_G" || namespace == "_ENV" =>
            {
                hostile_global(name, allow_dynamic_loaders)
            }
            _ => None,
        },
        _ => None,
    }
}

fn hostile_global(name: &str, allow_dynamic_loaders: bool) -> Option<&'static str> {
    match name {
        "load" if !allow_dynamic_loaders => Some("load"),
        "loadfile" if !allow_dynamic_loaders => Some("loadfile"),
        "dofile" if !allow_dynamic_loaders => Some("dofile"),
        "debug" => Some("debug"),
        _ => None,
    }
}
