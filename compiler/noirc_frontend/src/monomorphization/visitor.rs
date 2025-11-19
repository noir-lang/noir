use super::ast::{Expression, Ident, LValue, Literal};

/// Visit all identifiers under the [Expression].
pub fn visit_ident_mut<V>(expr: &mut Expression, v: &mut V)
where
    V: FnMut(&mut Ident),
{
    visit_expr_be_mut(expr, &mut |_| (true, ()), &mut |_, _| {}, v);
}

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
    visit_expr_be_mut(expr, &mut |e| (v(e), ()), &mut |_, _| {}, &mut |_| {});
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
///
/// It also takes a function to modify [Ident]s, which can be located in
/// `Expression::Ident` or `Expression::Assign`.
pub fn visit_expr_be_mut<B, E, T, I>(expr: &mut Expression, b: &mut B, e: &mut E, i: &mut I)
where
    B: FnMut(&mut Expression) -> (bool, T),
    E: FnMut(&Expression, T),
    I: FnMut(&mut Ident),
{
    let (go, token) = b(expr);
    if !go {
        return e(expr, token);
    }
    match expr {
        Expression::Ident(ident) => i(ident),
        Expression::Literal(literal) => match literal {
            Literal::Array(array_literal) => {
                for expr in array_literal.contents.iter_mut() {
                    visit_expr_be_mut(expr, b, e, i);
                }
            }
            Literal::Slice(array_literal) => {
                for expr in array_literal.contents.iter_mut() {
                    visit_expr_be_mut(expr, b, e, i);
                }
            }
            Literal::Integer(_, _, _) | Literal::Bool(_) | Literal::Unit | Literal::Str(_) => {}
            Literal::FmtStr(_, _, expr) => {
                visit_expr_be_mut(expr, b, e, i);
            }
        },
        Expression::Block(exprs) => {
            for expr in exprs.iter_mut() {
                visit_expr_be_mut(expr, b, e, i);
            }
        }
        Expression::Unary(unary) => {
            visit_expr_be_mut(&mut unary.rhs, b, e, i);
        }
        Expression::Binary(binary) => {
            visit_expr_be_mut(&mut binary.lhs, b, e, i);
            visit_expr_be_mut(&mut binary.rhs, b, e, i);
        }
        Expression::Index(index) => {
            visit_expr_be_mut(&mut index.collection, b, e, i);
            visit_expr_be_mut(&mut index.index, b, e, i);
        }
        Expression::Cast(cast) => {
            visit_expr_be_mut(&mut cast.lhs, b, e, i);
        }
        Expression::For(for_) => {
            visit_expr_be_mut(&mut for_.start_range, b, e, i);
            visit_expr_be_mut(&mut for_.end_range, b, e, i);
            visit_expr_be_mut(&mut for_.block, b, e, i);
        }
        Expression::Loop(expr) => {
            visit_expr_be_mut(expr, b, e, i);
        }
        Expression::While(while_) => {
            visit_expr_be_mut(&mut while_.condition, b, e, i);
            visit_expr_be_mut(&mut while_.body, b, e, i);
        }
        Expression::If(if_) => {
            visit_expr_be_mut(&mut if_.condition, b, e, i);
            visit_expr_be_mut(&mut if_.consequence, b, e, i);
            if let Some(ref mut alternative) = if_.alternative {
                visit_expr_be_mut(alternative, b, e, i);
            }
        }
        Expression::Match(match_) => {
            for case in match_.cases.iter_mut() {
                visit_expr_be_mut(&mut case.branch, b, e, i);
            }
            if let Some(ref mut case) = match_.default_case {
                visit_expr_be_mut(case, b, e, i);
            }
        }
        Expression::Tuple(exprs) => {
            for expr in exprs.iter_mut() {
                visit_expr_be_mut(expr, b, e, i);
            }
        }
        Expression::ExtractTupleField(expr, _) => {
            visit_expr_be_mut(expr, b, e, i);
        }
        Expression::Call(call) => {
            visit_expr_be_mut(&mut call.func, b, e, i);
            for arg in call.arguments.iter_mut() {
                visit_expr_be_mut(arg, b, e, i);
            }
        }
        Expression::Let(let_) => {
            visit_expr_be_mut(&mut let_.expression, b, e, i);
        }
        Expression::Constrain(expr, _, _) => {
            visit_expr_be_mut(expr, b, e, i);
        }
        Expression::Assign(assign) => {
            visit_lvalue_mut(&mut assign.lvalue, b, e, i);
            visit_expr_be_mut(&mut assign.expression, b, e, i);
        }
        Expression::Semi(expr) => {
            visit_expr_be_mut(expr, b, e, i);
        }
        Expression::Clone(expr) => {
            visit_expr_be_mut(expr, b, e, i);
        }
        Expression::Drop(expr) => {
            visit_expr_be_mut(expr, b, e, i);
        }
        Expression::Break => {}
        Expression::Continue => {}
    }

    e(expr, token);
}

fn visit_lvalue_mut<B, E, T, I>(lvalue: &mut LValue, b: &mut B, e: &mut E, i: &mut I)
where
    B: FnMut(&mut Expression) -> (bool, T),
    E: FnMut(&Expression, T),
    I: FnMut(&mut Ident),
{
    match lvalue {
        LValue::Ident(ident) => i(ident),
        LValue::Index { array, index, .. } => {
            visit_lvalue_mut(array.as_mut(), b, e, i);
            visit_expr_be_mut(index.as_mut(), b, e, i);
        }
        LValue::MemberAccess { object, .. } => {
            visit_lvalue_mut(object.as_mut(), b, e, i);
        }
        LValue::Dereference { reference, .. } => {
            visit_lvalue_mut(reference.as_mut(), b, e, i);
        }
        LValue::Clone(lvalue) => {
            visit_lvalue_mut(lvalue, b, e, i);
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
    visit_expr_be(expr, &mut |e| (v(e), ()), &mut |_, _| {}, &mut |_| {});
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
pub fn visit_expr_be<B, E, T, I>(expr: &Expression, b: &mut B, e: &mut E, i: &mut I)
where
    B: FnMut(&Expression) -> (bool, T),
    E: FnMut(&Expression, T),
    I: FnMut(&Ident),
{
    let (go, token) = b(expr);

    if !go {
        return e(expr, token);
    }

    match expr {
        Expression::Ident(ident) => i(ident),
        Expression::Literal(literal) => match literal {
            Literal::Array(array_literal) => {
                for expr in &array_literal.contents {
                    visit_expr_be(expr, b, e, i);
                }
            }
            Literal::Slice(array_literal) => {
                for expr in &array_literal.contents {
                    visit_expr_be(expr, b, e, i);
                }
            }
            Literal::Integer(_, _, _) | Literal::Bool(_) | Literal::Unit | Literal::Str(_) => {}
            Literal::FmtStr(_, _, expr) => {
                visit_expr_be(expr, b, e, i);
            }
        },
        Expression::Block(exprs) => {
            for expr in exprs {
                visit_expr_be(expr, b, e, i);
            }
        }
        Expression::Unary(unary) => {
            visit_expr_be(&unary.rhs, b, e, i);
        }
        Expression::Binary(binary) => {
            visit_expr_be(&binary.lhs, b, e, i);
            visit_expr_be(&binary.rhs, b, e, i);
        }
        Expression::Index(index) => {
            visit_expr_be(&index.collection, b, e, i);
            visit_expr_be(&index.index, b, e, i);
        }
        Expression::Cast(cast) => {
            visit_expr_be(&cast.lhs, b, e, i);
        }
        Expression::For(for_) => {
            visit_expr_be(&for_.start_range, b, e, i);
            visit_expr_be(&for_.end_range, b, e, i);
            visit_expr_be(&for_.block, b, e, i);
        }
        Expression::Loop(expr) => {
            visit_expr_be(expr, b, e, i);
        }
        Expression::While(while_) => {
            visit_expr_be(&while_.condition, b, e, i);
            visit_expr_be(&while_.body, b, e, i);
        }
        Expression::If(if_) => {
            visit_expr_be(&if_.condition, b, e, i);
            visit_expr_be(&if_.consequence, b, e, i);
            if let Some(ref alternative) = if_.alternative {
                visit_expr_be(alternative, b, e, i);
            }
        }
        Expression::Match(match_) => {
            for case in &match_.cases {
                visit_expr_be(&case.branch, b, e, i);
            }
            if let Some(ref case) = match_.default_case {
                visit_expr_be(case, b, e, i);
            }
        }
        Expression::Tuple(exprs) => {
            for expr in exprs {
                visit_expr_be(expr, b, e, i);
            }
        }
        Expression::ExtractTupleField(expr, _) => {
            visit_expr_be(expr, b, e, i);
        }
        Expression::Call(call) => {
            visit_expr_be(&call.func, b, e, i);
            for arg in &call.arguments {
                visit_expr_be(arg, b, e, i);
            }
        }
        Expression::Let(let_) => {
            visit_expr_be(&let_.expression, b, e, i);
        }
        Expression::Constrain(expr, _, _) => {
            visit_expr_be(expr, b, e, i);
        }
        Expression::Assign(assign) => {
            visit_lvalue(&assign.lvalue, b, e, i);
            visit_expr_be(&assign.expression, b, e, i);
        }
        Expression::Semi(expr) => {
            visit_expr_be(expr, b, e, i);
        }
        Expression::Clone(expr) => {
            visit_expr_be(expr, b, e, i);
        }
        Expression::Drop(expr) => {
            visit_expr_be(expr, b, e, i);
        }
        Expression::Break => {}
        Expression::Continue => {}
    }

    e(expr, token);
}

fn visit_lvalue<B, E, T, I>(lvalue: &LValue, b: &mut B, e: &mut E, i: &mut I)
where
    B: FnMut(&Expression) -> (bool, T),
    E: FnMut(&Expression, T),
    I: FnMut(&Ident),
{
    match lvalue {
        LValue::Ident(ident) => i(ident),
        LValue::Index { array, index, .. } => {
            visit_lvalue(array.as_ref(), b, e, i);
            visit_expr_be(index.as_ref(), b, e, i);
        }
        LValue::MemberAccess { object, .. } => {
            visit_lvalue(object.as_ref(), b, e, i);
        }
        LValue::Dereference { reference, .. } => {
            visit_lvalue(reference.as_ref(), b, e, i);
        }
        LValue::Clone(lvalue) => {
            visit_lvalue(lvalue, b, e, i);
        }
    }
}
