use noirc_frontend::monomorphization::ast::{Expression, LValue, Literal};

/// Visit for the contents of an [Expression] representing the AST.
///
/// The `bool` returned indicates whether we want to visit the children
/// of the visited expression.
///
/// Gets mutable references so it can manipulate the expressions if needed.
pub(crate) fn visit_expr_mut<V>(expr: &mut Expression, mut visit: V)
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
                    visit_expr_mut(expr, &mut visit);
                }
            }
            Literal::Slice(array_literal) => {
                for expr in array_literal.contents.iter_mut() {
                    visit_expr_mut(expr, &mut visit);
                }
            }
            Literal::Integer(_, _, _) | Literal::Bool(_) | Literal::Unit | Literal::Str(_) => {}
            Literal::FmtStr(_, _, expr) => {
                visit_expr_mut(expr, &mut visit);
            }
        },
        Expression::Block(expressions) => {
            for expr in expressions.iter_mut() {
                visit_expr_mut(expr, &mut visit);
            }
        }
        Expression::Unary(unary) => {
            visit_expr_mut(&mut unary.rhs, &mut visit);
        }
        Expression::Binary(binary) => {
            visit_expr_mut(&mut binary.lhs, &mut visit);
            visit_expr_mut(&mut binary.rhs, &mut visit);
        }
        Expression::Index(index) => {
            visit_expr_mut(&mut index.collection, &mut visit);
            visit_expr_mut(&mut index.index, &mut visit);
        }
        Expression::Cast(cast) => {
            visit_expr_mut(&mut cast.lhs, &mut visit);
        }
        Expression::For(for_) => {
            visit_expr_mut(&mut for_.start_range, &mut visit);
            visit_expr_mut(&mut for_.end_range, &mut visit);
            visit_expr_mut(&mut for_.block, &mut visit);
        }
        Expression::Loop(expr) => {
            visit_expr_mut(expr, &mut visit);
        }
        Expression::While(while_) => {
            visit_expr_mut(&mut while_.condition, &mut visit);
            visit_expr_mut(&mut while_.body, &mut visit);
        }
        Expression::If(if_) => {
            visit_expr_mut(&mut if_.condition, &mut visit);
            visit_expr_mut(&mut if_.consequence, &mut visit);
            if let Some(ref mut alternative) = if_.alternative {
                visit_expr_mut(alternative, &mut visit);
            }
        }
        Expression::Match(match_) => {
            for case in match_.cases.iter_mut() {
                visit_expr_mut(&mut case.branch, &mut visit);
            }
            if let Some(ref mut case) = match_.default_case {
                visit_expr_mut(case, &mut visit);
            }
        }
        Expression::Tuple(expressions) => {
            for expr in expressions.iter_mut() {
                visit_expr_mut(expr, &mut visit);
            }
        }
        Expression::ExtractTupleField(expression, _) => {
            visit_expr_mut(expr, &mut visit);
        }
        Expression::Call(call) => {
            visit_expr_mut(&mut call.func, &mut visit);
            for arg in call.arguments.iter_mut() {
                visit_expr_mut(arg, &mut visit);
            }
        }
        Expression::Let(let_) => {
            visit_expr_mut(&mut let_.expression, &mut visit);
        }
        Expression::Constrain(expr, _, _) => {
            visit_expr_mut(expr, &mut visit);
        }
        Expression::Assign(assign) => {
            visit_lvalue_mut(&mut assign.lvalue, &mut visit);
            visit_expr_mut(&mut assign.expression, &mut visit);
        }
        Expression::Semi(expr) => {
            visit_expr_mut(expr, &mut visit);
        }
        Expression::Clone(expr) => {
            visit_expr_mut(expr, &mut visit);
        }
        Expression::Drop(expr) => {
            visit_expr_mut(expr, &mut visit);
        }
        Expression::Break => {}
        Expression::Continue => {}
    }
}

fn visit_lvalue_mut<V>(lvalue: &mut LValue, mut visit: V)
where
    V: FnMut(&mut Expression) -> bool,
{
    match lvalue {
        LValue::Ident(_) => {}
        LValue::Index { array, index, .. } => {
            visit_lvalue_mut(array.as_mut(), &mut visit);
            visit_expr_mut(index.as_mut(), &mut visit);
        }
        LValue::MemberAccess { object, .. } => {
            visit_lvalue_mut(object.as_mut(), &mut visit);
        }
        LValue::Dereference { reference, .. } => {
            visit_lvalue_mut(reference.as_mut(), &mut visit);
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
pub(crate) fn visit_expr<V>(expr: &Expression, mut visit: V)
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
                for expr in array_literal.contents.iter() {
                    visit_expr(expr, &mut visit);
                }
            }
            Literal::Slice(array_literal) => {
                for expr in array_literal.contents.iter() {
                    visit_expr(expr, &mut visit);
                }
            }
            Literal::Integer(_, _, _) | Literal::Bool(_) | Literal::Unit | Literal::Str(_) => {}
            Literal::FmtStr(_, _, expr) => {
                visit_expr(expr, &mut visit);
            }
        },
        Expression::Block(expressions) => {
            for expr in expressions.iter() {
                visit_expr(expr, &mut visit);
            }
        }
        Expression::Unary(unary) => {
            visit_expr(&unary.rhs, &mut visit);
        }
        Expression::Binary(binary) => {
            visit_expr(&binary.lhs, &mut visit);
            visit_expr(&binary.rhs, &mut visit);
        }
        Expression::Index(index) => {
            visit_expr(&index.collection, &mut visit);
            visit_expr(&index.index, &mut visit);
        }
        Expression::Cast(cast) => {
            visit_expr(&cast.lhs, &mut visit);
        }
        Expression::For(for_) => {
            visit_expr(&for_.start_range, &mut visit);
            visit_expr(&for_.end_range, &mut visit);
            visit_expr(&for_.block, &mut visit);
        }
        Expression::Loop(expr) => {
            visit_expr(expr, &mut visit);
        }
        Expression::While(while_) => {
            visit_expr(&while_.condition, &mut visit);
            visit_expr(&while_.body, &mut visit);
        }
        Expression::If(if_) => {
            visit_expr(&if_.condition, &mut visit);
            visit_expr(&if_.consequence, &mut visit);
            if let Some(ref alternative) = if_.alternative {
                visit_expr(alternative, &mut visit);
            }
        }
        Expression::Match(match_) => {
            for case in match_.cases.iter() {
                visit_expr(&case.branch, &mut visit);
            }
            if let Some(ref case) = match_.default_case {
                visit_expr(case, &mut visit);
            }
        }
        Expression::Tuple(expressions) => {
            for expr in expressions.iter() {
                visit_expr(expr, &mut visit);
            }
        }
        Expression::ExtractTupleField(expression, _) => {
            visit_expr(expr, &mut visit);
        }
        Expression::Call(call) => {
            visit_expr(&call.func, &mut visit);
            for arg in call.arguments.iter() {
                visit_expr(arg, &mut visit);
            }
        }
        Expression::Let(let_) => {
            visit_expr(&let_.expression, &mut visit);
        }
        Expression::Constrain(expr, _, _) => {
            visit_expr(expr, &mut visit);
        }
        Expression::Assign(assign) => {
            visit_lvalue(&assign.lvalue, &mut visit);
            visit_expr(&assign.expression, &mut visit);
        }
        Expression::Semi(expr) => {
            visit_expr(expr, &mut visit);
        }
        Expression::Clone(expr) => {
            visit_expr(expr, &mut visit);
        }
        Expression::Drop(expr) => {
            visit_expr(expr, &mut visit);
        }
        Expression::Break => {}
        Expression::Continue => {}
    }
}

fn visit_lvalue<V>(lvalue: &LValue, mut visit: V)
where
    V: FnMut(&Expression) -> bool,
{
    match lvalue {
        LValue::Ident(_) => {}
        LValue::Index { array, index, .. } => {
            visit_lvalue(array.as_ref(), &mut visit);
            visit_expr(index.as_ref(), &mut visit);
        }
        LValue::MemberAccess { object, .. } => {
            visit_lvalue(object.as_ref(), visit);
        }
        LValue::Dereference { reference, .. } => {
            visit_lvalue(reference.as_ref(), visit);
        }
    }
}
