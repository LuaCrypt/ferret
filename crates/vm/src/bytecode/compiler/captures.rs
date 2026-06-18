use std::collections::BTreeSet;

use ferret_ir::{Expr, Stmt};

pub(super) fn collect_captures(
    params: &[String],
    body: &[Stmt],
    available: &BTreeSet<String>,
) -> Vec<String> {
    let mut bound = params.iter().cloned().collect();
    let mut captures = BTreeSet::new();
    stmts(body, available, &mut bound, &mut captures);
    captures.into_iter().collect()
}

fn stmts(
    stmts_in: &[Stmt],
    available: &BTreeSet<String>,
    bound: &mut BTreeSet<String>,
    captures: &mut BTreeSet<String>,
) {
    for stmt in stmts_in {
        stmt_refs(stmt, available, bound, captures);
    }
}

fn stmt_refs(
    stmt: &Stmt,
    available: &BTreeSet<String>,
    bound: &mut BTreeSet<String>,
    captures: &mut BTreeSet<String>,
) {
    match stmt {
        Stmt::Local { names, values } => {
            for value in values {
                expr(value, available, bound, captures);
            }
            bound.extend(names.iter().cloned());
        }
        Stmt::Assign { targets, values } => {
            for target in targets {
                expr(target, available, bound, captures);
            }
            for value in values {
                expr(value, available, bound, captures);
            }
        }
        Stmt::Block(body) => stmts(body, available, bound, captures),
        Stmt::Break | Stmt::Label(_) | Stmt::Goto(_) => {}
        Stmt::Expr(value) => expr(value, available, bound, captures),
        Stmt::If {
            cond,
            then_body,
            else_body,
        } => {
            expr(cond, available, bound, captures);
            stmts(then_body, available, &mut bound.clone(), captures);
            stmts(else_body, available, &mut bound.clone(), captures);
        }
        Stmt::While { cond, body } | Stmt::Repeat { cond, body } => {
            expr(cond, available, bound, captures);
            stmts(body, available, &mut bound.clone(), captures);
        }
        Stmt::NumericFor {
            name,
            start,
            end,
            step,
            body,
        } => {
            expr(start, available, bound, captures);
            expr(end, available, bound, captures);
            expr(step, available, bound, captures);
            let mut inner = bound.clone();
            inner.insert(name.clone());
            stmts(body, available, &mut inner, captures);
        }
        Stmt::GenericFor { names, iter, body } => {
            for value in iter {
                expr(value, available, bound, captures);
            }
            let mut inner = bound.clone();
            inner.extend(names.iter().cloned());
            stmts(body, available, &mut inner, captures);
        }
        Stmt::Return(values) => {
            for value in values {
                expr(value, available, bound, captures);
            }
        }
    }
}

fn expr(
    value: &Expr,
    available: &BTreeSet<String>,
    bound: &BTreeSet<String>,
    captures: &mut BTreeSet<String>,
) {
    match value {
        Expr::Var(name) => {
            if !bound.contains(name) {
                if available.contains(name) {
                    captures.insert(name.clone());
                } else if name != "_ENV" && available.contains("_ENV") && !bound.contains("_ENV") {
                    captures.insert("_ENV".to_string());
                }
            }
        }
        Expr::Table(fields) => {
            for (key, value) in fields {
                if let Some(key) = key {
                    expr(key, available, bound, captures);
                }
                expr(value, available, bound, captures);
            }
        }
        Expr::Unary { expr: inner, .. } => expr(inner, available, bound, captures),
        Expr::Binary { left, right, .. } => {
            expr(left, available, bound, captures);
            expr(right, available, bound, captures);
        }
        Expr::Call { callee, args } => {
            expr(callee, available, bound, captures);
            for arg in args {
                expr(arg, available, bound, captures);
            }
        }
        Expr::Function {
            params,
            vararg: _,
            body,
        } => {
            let mut inner = bound.clone();
            inner.extend(params.iter().cloned());
            stmts(body, available, &mut inner, captures);
        }
        Expr::Index { table, key } => {
            expr(table, available, bound, captures);
            expr(key, available, bound, captures);
        }
        Expr::Nil | Expr::Bool(_) | Expr::Number(_) | Expr::String(_) | Expr::VarArgs => {}
    }
}
