use noirc_frontend::monomorphization::ast::{Expression, LValue, Literal};

/// Visit the contents of an [Expression] representing the AST.
///
/// The `bool` returned indicates whether we want to visit the children
/// of the visited expression.
///
/// Gets mutable references so it can manipulate the expressions if needed.
pub fn visit_expr_mut<V>(expr: &mut Expression, v: &mut V)
where
    V: FnMut(&mut Expression) -> bool,
{
    visit_expr_be_mut(expr, &mut |e| (v(e), ()), &mut |_, _| {});
}

/// Visit for the contents of an [Expression] representing the AST,
/// passing each to a _begin_ and _end_ function.
///
/// The `bool` returned from _begin_ indicates whether we want to
/// visit the children of the visited expression.
///
/// Gets mutable references so it can manipulate the expressions if needed.
///
/// Compared to [visit_expr_mut], this version allows the caller to maintain
/// scopes and context, facilitated by a _token_ passed between _begin_ and _end_.
pub fn visit_expr_be_mut<B, E, T>(expr: &mut Expression, b: &mut B, e: &mut E)
where
    B: FnMut(&mut Expression) -> (bool, T),
    E: FnMut(&Expression, T),
{
    let (go, token) = b(expr);
    if !go {
        return e(expr, token);
    }
    match expr {
        Expression::Ident(_) => {}
        Expression::Literal(literal) => match literal {
            Literal::Array(array_literal) => {
                for expr in array_literal.contents.iter_mut() {
                    visit_expr_be_mut(expr, b, e);
                }
            }
            Literal::Slice(array_literal) => {
                for expr in array_literal.contents.iter_mut() {
                    visit_expr_be_mut(expr, b, e);
                }
            }
            Literal::Integer(_, _, _) | Literal::Bool(_) | Literal::Unit | Literal::Str(_) => {}
            Literal::FmtStr(_, _, expr) => {
                visit_expr_be_mut(expr, b, e);
            }
        },
        Expression::Block(exprs) => {
            for expr in exprs.iter_mut() {
                visit_expr_be_mut(expr, b, e);
            }
        }
        Expression::Unary(unary) => {
            visit_expr_be_mut(&mut unary.rhs, b, e);
        }
        Expression::Binary(binary) => {
            visit_expr_be_mut(&mut binary.lhs, b, e);
            visit_expr_be_mut(&mut binary.rhs, b, e);
        }
        Expression::Index(index) => {
            visit_expr_be_mut(&mut index.collection, b, e);
            visit_expr_be_mut(&mut index.index, b, e);
        }
        Expression::Cast(cast) => {
            visit_expr_be_mut(&mut cast.lhs, b, e);
        }
        Expression::For(for_) => {
            visit_expr_be_mut(&mut for_.start_range, b, e);
            visit_expr_be_mut(&mut for_.end_range, b, e);
            visit_expr_be_mut(&mut for_.block, b, e);
        }
        Expression::Loop(expr) => {
            visit_expr_be_mut(expr, b, e);
        }
        Expression::While(while_) => {
            visit_expr_be_mut(&mut while_.condition, b, e);
            visit_expr_be_mut(&mut while_.body, b, e);
        }
        Expression::If(if_) => {
            visit_expr_be_mut(&mut if_.condition, b, e);
            visit_expr_be_mut(&mut if_.consequence, b, e);
            if let Some(ref mut alternative) = if_.alternative {
                visit_expr_be_mut(alternative, b, e);
            }
        }
        Expression::Match(match_) => {
            for case in match_.cases.iter_mut() {
                visit_expr_be_mut(&mut case.branch, b, e);
            }
            if let Some(ref mut case) = match_.default_case {
                visit_expr_be_mut(case, b, e);
            }
        }
        Expression::Tuple(exprs) => {
            for expr in exprs.iter_mut() {
                visit_expr_be_mut(expr, b, e);
            }
        }
        Expression::ExtractTupleField(expr, _) => {
            visit_expr_be_mut(expr, b, e);
        }
        Expression::Call(call) => {
            visit_expr_be_mut(&mut call.func, b, e);
            for arg in call.arguments.iter_mut() {
                visit_expr_be_mut(arg, b, e);
            }
        }
        Expression::Let(let_) => {
            visit_expr_be_mut(&mut let_.expression, b, e);
        }
        Expression::Constrain(expr, _, _) => {
            visit_expr_be_mut(expr, b, e);
        }
        Expression::Assign(assign) => {
            visit_lvalue_mut(&mut assign.lvalue, b, e);
            visit_expr_be_mut(&mut assign.expression, b, e);
        }
        Expression::Semi(expr) => {
            visit_expr_be_mut(expr, b, e);
        }
        Expression::Clone(expr) => {
            visit_expr_be_mut(expr, b, e);
        }
        Expression::Drop(expr) => {
            visit_expr_be_mut(expr, b, e);
        }
        Expression::Break => {}
        Expression::Continue => {}
    }

    e(expr, token);
}

fn visit_lvalue_mut<B, E, T>(lvalue: &mut LValue, b: &mut B, e: &mut E)
where
    B: FnMut(&mut Expression) -> (bool, T),
    E: FnMut(&Expression, T),
{
    match lvalue {
        LValue::Ident(_) => {}
        LValue::Index { array, index, .. } => {
            visit_lvalue_mut(array.as_mut(), b, e);
            visit_expr_be_mut(index.as_mut(), b, e);
        }
        LValue::MemberAccess { object, .. } => {
            visit_lvalue_mut(object.as_mut(), b, e);
        }
        LValue::Dereference { reference, .. } => {
            visit_lvalue_mut(reference.as_mut(), b, e);
        }
    }
}

/// Visit the contents of an [Expression] representing the AST.
///
/// The `bool` returned indicates whether we want to visit the children
/// of the visited expression.
///
/// This is a read-only version of [visit_expr_mut], for cases where
/// we don't have/need a mutable reference to the AST.
pub fn visit_expr<V>(expr: &Expression, v: &mut V)
where
    V: FnMut(&Expression) -> bool,
{
    visit_expr_be(expr, &mut |e| (v(e), ()), &mut |_, _| {});
}

/// Visit the contents of an [Expression] representing the AST,
/// passing them to a _begin_ and _end_ function.
///
/// The `bool` returned by _begin_ indicates whether we want to
/// visit the children of the visited expression.
///
/// This is a read-only version [visit_expr_be_mut], for cases where
/// we don't have/need a mutable reference to the AST.
///
/// Compared to [visit_expr], this version allows the caller to maintain
/// scopes and context, facilitated by a _token_ passed between _begin_ and _end_.
pub fn visit_expr_be<B, E, T>(expr: &Expression, b: &mut B, e: &mut E)
where
    B: FnMut(&Expression) -> (bool, T),
    E: FnMut(&Expression, T),
{
    let (go, token) = b(expr);

    if !go {
        return e(expr, token);
    }

    match expr {
        Expression::Ident(_) => {}
        Expression::Literal(literal) => match literal {
            Literal::Array(array_literal) => {
                for expr in &array_literal.contents {
                    visit_expr_be(expr, b, e);
                }
            }
            Literal::Slice(array_literal) => {
                for expr in &array_literal.contents {
                    visit_expr_be(expr, b, e);
                }
            }
            Literal::Integer(_, _, _) | Literal::Bool(_) | Literal::Unit | Literal::Str(_) => {}
            Literal::FmtStr(_, _, expr) => {
                visit_expr_be(expr, b, e);
            }
        },
        Expression::Block(exprs) => {
            for expr in exprs {
                visit_expr_be(expr, b, e);
            }
        }
        Expression::Unary(unary) => {
            visit_expr_be(&unary.rhs, b, e);
        }
        Expression::Binary(binary) => {
            visit_expr_be(&binary.lhs, b, e);
            visit_expr_be(&binary.rhs, b, e);
        }
        Expression::Index(index) => {
            visit_expr_be(&index.collection, b, e);
            visit_expr_be(&index.index, b, e);
        }
        Expression::Cast(cast) => {
            visit_expr_be(&cast.lhs, b, e);
        }
        Expression::For(for_) => {
            visit_expr_be(&for_.start_range, b, e);
            visit_expr_be(&for_.end_range, b, e);
            visit_expr_be(&for_.block, b, e);
        }
        Expression::Loop(expr) => {
            visit_expr_be(expr, b, e);
        }
        Expression::While(while_) => {
            visit_expr_be(&while_.condition, b, e);
            visit_expr_be(&while_.body, b, e);
        }
        Expression::If(if_) => {
            visit_expr_be(&if_.condition, b, e);
            visit_expr_be(&if_.consequence, b, e);
            if let Some(ref alternative) = if_.alternative {
                visit_expr_be(alternative, b, e);
            }
        }
        Expression::Match(match_) => {
            for case in &match_.cases {
                visit_expr_be(&case.branch, b, e);
            }
            if let Some(ref case) = match_.default_case {
                visit_expr_be(case, b, e);
            }
        }
        Expression::Tuple(exprs) => {
            for expr in exprs {
                visit_expr_be(expr, b, e);
            }
        }
        Expression::ExtractTupleField(expr, _) => {
            visit_expr_be(expr, b, e);
        }
        Expression::Call(call) => {
            visit_expr_be(&call.func, b, e);
            for arg in &call.arguments {
                visit_expr_be(arg, b, e);
            }
        }
        Expression::Let(let_) => {
            visit_expr_be(&let_.expression, b, e);
        }
        Expression::Constrain(expr, _, _) => {
            visit_expr_be(expr, b, e);
        }
        Expression::Assign(assign) => {
            visit_lvalue(&assign.lvalue, b, e);
            visit_expr_be(&assign.expression, b, e);
        }
        Expression::Semi(expr) => {
            visit_expr_be(expr, b, e);
        }
        Expression::Clone(expr) => {
            visit_expr_be(expr, b, e);
        }
        Expression::Drop(expr) => {
            visit_expr_be(expr, b, e);
        }
        Expression::Break => {}
        Expression::Continue => {}
    }

    e(expr, token);
}

fn visit_lvalue<B, E, T>(lvalue: &LValue, b: &mut B, e: &mut E)
where
    B: FnMut(&Expression) -> (bool, T),
    E: FnMut(&Expression, T),
{
    match lvalue {
        LValue::Ident(_) => {}
        LValue::Index { array, index, .. } => {
            visit_lvalue(array.as_ref(), b, e);
            visit_expr_be(index.as_ref(), b, e);
        }
        LValue::MemberAccess { object, .. } => {
            visit_lvalue(object.as_ref(), b, e);
        }
        LValue::Dereference { reference, .. } => {
            visit_lvalue(reference.as_ref(), b, e);
        }
    }
}
