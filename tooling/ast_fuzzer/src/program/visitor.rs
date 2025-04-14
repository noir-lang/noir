use noirc_frontend::monomorphization::ast::{Expression, LValue, Literal};

/// Visit for the contents of an [Expression] representing the AST.
///
/// The `bool` returned indicates whether we want to visit the children
/// of the visited expression.
///
/// Gets mutable references so it can manipulate the expressions if needed.
pub(crate) fn visit_expr_mut<V>(expr: &mut Expression, visit: &mut V)
where
    V: FnMut(&mut Expression) -> bool,
{
    if !visit(expr) {
        return;
    }
    match expr {
        Expression::Ident(_) => {}
        Expression::Literal(literal) => match literal {
            Literal::Array(array_literal) => {
                for expr in array_literal.contents.iter_mut() {
                    visit_expr_mut(expr, visit);
                }
            }
            Literal::Slice(array_literal) => {
                for expr in array_literal.contents.iter_mut() {
                    visit_expr_mut(expr, visit);
                }
            }
            Literal::Integer(_, _, _) | Literal::Bool(_) | Literal::Unit | Literal::Str(_) => {}
            Literal::FmtStr(_, _, expr) => {
                visit_expr_mut(expr, visit);
            }
        },
        Expression::Block(exprs) => {
            for expr in exprs.iter_mut() {
                visit_expr_mut(expr, visit);
            }
        }
        Expression::Unary(unary) => {
            visit_expr_mut(&mut unary.rhs, visit);
        }
        Expression::Binary(binary) => {
            visit_expr_mut(&mut binary.lhs, visit);
            visit_expr_mut(&mut binary.rhs, visit);
        }
        Expression::Index(index) => {
            visit_expr_mut(&mut index.collection, visit);
            visit_expr_mut(&mut index.index, visit);
        }
        Expression::Cast(cast) => {
            visit_expr_mut(&mut cast.lhs, visit);
        }
        Expression::For(for_) => {
            visit_expr_mut(&mut for_.start_range, visit);
            visit_expr_mut(&mut for_.end_range, visit);
            visit_expr_mut(&mut for_.block, visit);
        }
        Expression::Loop(expr) => {
            visit_expr_mut(expr, visit);
        }
        Expression::While(while_) => {
            visit_expr_mut(&mut while_.condition, visit);
            visit_expr_mut(&mut while_.body, visit);
        }
        Expression::If(if_) => {
            visit_expr_mut(&mut if_.condition, visit);
            visit_expr_mut(&mut if_.consequence, visit);
            if let Some(ref mut alternative) = if_.alternative {
                visit_expr_mut(alternative, visit);
            }
        }
        Expression::Match(match_) => {
            for case in match_.cases.iter_mut() {
                visit_expr_mut(&mut case.branch, visit);
            }
            if let Some(ref mut case) = match_.default_case {
                visit_expr_mut(case, visit);
            }
        }
        Expression::Tuple(exprs) => {
            for expr in exprs.iter_mut() {
                visit_expr_mut(expr, visit);
            }
        }
        Expression::ExtractTupleField(expr, _) => {
            visit_expr_mut(expr, visit);
        }
        Expression::Call(call) => {
            visit_expr_mut(&mut call.func, visit);
            for arg in call.arguments.iter_mut() {
                visit_expr_mut(arg, visit);
            }
        }
        Expression::Let(let_) => {
            visit_expr_mut(&mut let_.expression, visit);
        }
        Expression::Constrain(expr, _, _) => {
            visit_expr_mut(expr, visit);
        }
        Expression::Assign(assign) => {
            visit_lvalue_mut(&mut assign.lvalue, visit);
            visit_expr_mut(&mut assign.expression, visit);
        }
        Expression::Semi(expr) => {
            visit_expr_mut(expr, visit);
        }
        Expression::Clone(expr) => {
            visit_expr_mut(expr, visit);
        }
        Expression::Drop(expr) => {
            visit_expr_mut(expr, visit);
        }
        Expression::Break => {}
        Expression::Continue => {}
    }
}

fn visit_lvalue_mut<V>(lvalue: &mut LValue, visit: &mut V)
where
    V: FnMut(&mut Expression) -> bool,
{
    match lvalue {
        LValue::Ident(_) => {}
        LValue::Index { array, index, .. } => {
            visit_lvalue_mut(array.as_mut(), visit);
            visit_expr_mut(index.as_mut(), visit);
        }
        LValue::MemberAccess { object, .. } => {
            visit_lvalue_mut(object.as_mut(), visit);
        }
        LValue::Dereference { reference, .. } => {
            visit_lvalue_mut(reference.as_mut(), visit);
        }
    }
}

/// Visit for the contents of an [Expression] representing the AST.
///
/// The `bool` returned indicates whether we want to visit the children
/// of the visited expression.
///
/// This is a read-only version, for cases where we don't have/need
/// a mutable reference to the AST.
pub(crate) fn visit_expr<V>(expr: &Expression, visit: &mut V)
where
    V: FnMut(&Expression) -> bool,
{
    if !visit(expr) {
        return;
    }
    match expr {
        Expression::Ident(_) => {}
        Expression::Literal(literal) => match literal {
            Literal::Array(array_literal) => {
                for expr in &array_literal.contents {
                    visit_expr(expr, visit);
                }
            }
            Literal::Slice(array_literal) => {
                for expr in &array_literal.contents {
                    visit_expr(expr, visit);
                }
            }
            Literal::Integer(_, _, _) | Literal::Bool(_) | Literal::Unit | Literal::Str(_) => {}
            Literal::FmtStr(_, _, expr) => {
                visit_expr(expr, visit);
            }
        },
        Expression::Block(exprs) => {
            for expr in exprs {
                visit_expr(expr, visit);
            }
        }
        Expression::Unary(unary) => {
            visit_expr(&unary.rhs, visit);
        }
        Expression::Binary(binary) => {
            visit_expr(&binary.lhs, visit);
            visit_expr(&binary.rhs, visit);
        }
        Expression::Index(index) => {
            visit_expr(&index.collection, visit);
            visit_expr(&index.index, visit);
        }
        Expression::Cast(cast) => {
            visit_expr(&cast.lhs, visit);
        }
        Expression::For(for_) => {
            visit_expr(&for_.start_range, visit);
            visit_expr(&for_.end_range, visit);
            visit_expr(&for_.block, visit);
        }
        Expression::Loop(expr) => {
            visit_expr(expr, visit);
        }
        Expression::While(while_) => {
            visit_expr(&while_.condition, visit);
            visit_expr(&while_.body, visit);
        }
        Expression::If(if_) => {
            visit_expr(&if_.condition, visit);
            visit_expr(&if_.consequence, visit);
            if let Some(ref alternative) = if_.alternative {
                visit_expr(alternative, visit);
            }
        }
        Expression::Match(match_) => {
            for case in &match_.cases {
                visit_expr(&case.branch, visit);
            }
            if let Some(ref case) = match_.default_case {
                visit_expr(case, visit);
            }
        }
        Expression::Tuple(exprs) => {
            for expr in exprs {
                visit_expr(expr, visit);
            }
        }
        Expression::ExtractTupleField(expr, _) => {
            visit_expr(expr, visit);
        }
        Expression::Call(call) => {
            visit_expr(&call.func, visit);
            for arg in &call.arguments {
                visit_expr(arg, visit);
            }
        }
        Expression::Let(let_) => {
            visit_expr(&let_.expression, visit);
        }
        Expression::Constrain(expr, _, _) => {
            visit_expr(expr, visit);
        }
        Expression::Assign(assign) => {
            visit_lvalue(&assign.lvalue, visit);
            visit_expr(&assign.expression, visit);
        }
        Expression::Semi(expr) => {
            visit_expr(expr, visit);
        }
        Expression::Clone(expr) => {
            visit_expr(expr, visit);
        }
        Expression::Drop(expr) => {
            visit_expr(expr, visit);
        }
        Expression::Break => {}
        Expression::Continue => {}
    }
}

fn visit_lvalue<V>(lvalue: &LValue, visit: &mut V)
where
    V: FnMut(&Expression) -> bool,
{
    match lvalue {
        LValue::Ident(_) => {}
        LValue::Index { array, index, .. } => {
            visit_lvalue(array.as_ref(), visit);
            visit_expr(index.as_ref(), visit);
        }
        LValue::MemberAccess { object, .. } => {
            visit_lvalue(object.as_ref(), visit);
        }
        LValue::Dereference { reference, .. } => {
            visit_lvalue(reference.as_ref(), visit);
        }
    }
}
