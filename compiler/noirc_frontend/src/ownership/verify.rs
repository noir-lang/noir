//! Post-condition verifier for the ownership pass.
//!
//! After the ownership pass inserts `Clone` nodes, this module walks
//! every unconstrained function and checks the following invariant:
//!
//! **No array-typed local variable is used as a bare (non-cloned) identifier
//! after it has already been consumed (used as a bare identifier) on any
//! reachable execution path.**
//!
//! A "bare use" means the identifier appears as `Expression::Ident` in a
//! *consuming* position (not wrapped in `Expression::Clone` and not in a
//! *reference* position like an index collection or dereference operand).
//! After the ownership pass, only the last use of a variable should be bare;
//! every earlier use should have been wrapped in a `Clone`. If this invariant
//! is violated it indicates a missing clone.
//!
//! This verification only runs in debug builds (`#[cfg(debug_assertions)]`).

use crate::ast::UnaryOp;
use crate::monomorphization::ast::{
    Definition, Expression, Function, LValue, Literal, LocalId, Unary,
};

use rustc_hash::FxHashSet;

use super::contains_array_or_str_type;

/// Set of local variables that have been consumed (used bare) on the current path.
#[derive(Clone, Default)]
struct MoveState {
    possibly_moved: FxHashSet<LocalId>,
}

impl MoveState {
    /// Merge another state into this one (union of possibly-moved sets).
    fn union(&mut self, other: &MoveState) {
        for id in &other.possibly_moved {
            self.possibly_moved.insert(*id);
        }
    }

    fn mark_moved(&mut self, id: LocalId) {
        self.possibly_moved.insert(id);
    }

    fn mark_fresh(&mut self, id: LocalId) {
        self.possibly_moved.remove(&id);
    }

    fn is_moved(&self, id: LocalId) -> bool {
        self.possibly_moved.contains(&id)
    }
}

/// Entry point: verify that no use-after-move exists in the given function.
///
/// Panics with a diagnostic message if a violation is found.
pub(super) fn verify_no_use_after_move(function: &Function) {
    if !function.unconstrained {
        return;
    }

    let mut state = MoveState::default();
    verify_expression(&function.body, &mut state, &function.name);
}

/// Walk an expression, tracking moves.
///
/// An `Expression::Ident` for an array-typed local in this context is a
/// *consuming* use: it asserts the variable hasn't been moved and then marks
/// it as moved.
fn verify_expression(expr: &Expression, state: &mut MoveState, fn_name: &str) {
    match expr {
        Expression::Ident(ident) => {
            if let Definition::Local(local_id) = &ident.definition
                && contains_array_or_str_type(&ident.typ)
            {
                assert!(
                    !state.is_moved(*local_id),
                    "Use-after-move in function `{fn_name}`: variable `{}` (local {:?}) \
                     used after being consumed without a clone",
                    ident.name,
                    local_id,
                );
                state.mark_moved(*local_id);
            }
        }

        Expression::Clone(inner) => {
            // Cloned expression: not consumed. But verify inner is still valid.
            verify_non_consuming(inner, state, fn_name);
        }

        Expression::Drop(inner) => {
            // Drop doesn't consume; just verify inner is valid.
            verify_non_consuming(inner, state, fn_name);
        }

        Expression::Block(exprs) => {
            for e in exprs {
                verify_expression(e, state, fn_name);
            }
        }

        Expression::Let(let_expr) => {
            verify_expression(&let_expr.expression, state, fn_name);
            state.mark_fresh(let_expr.id);
        }

        Expression::Assign(assign) => {
            verify_expression(&assign.expression, state, fn_name);
            verify_lvalue(&assign.lvalue, state, fn_name);
            if let LValue::Ident(ident) = &assign.lvalue
                && let Definition::Local(local_id) = &ident.definition
            {
                state.mark_fresh(*local_id);
            }
        }

        Expression::If(if_expr) => {
            verify_expression(&if_expr.condition, state, fn_name);

            let mut then_state = state.clone();
            verify_expression(&if_expr.consequence, &mut then_state, fn_name);

            let mut else_state = state.clone();
            if let Some(alt) = &if_expr.alternative {
                verify_expression(alt, &mut else_state, fn_name);
            }

            then_state.union(&else_state);
            *state = then_state;
        }

        Expression::Match(match_expr) => {
            let mut merged = state.clone();
            let mut first = true;

            for case in &match_expr.cases {
                let mut case_state = state.clone();
                for (arg_id, _) in &case.arguments {
                    case_state.mark_fresh(*arg_id);
                }
                verify_expression(&case.branch, &mut case_state, fn_name);
                if first {
                    merged = case_state;
                    first = false;
                } else {
                    merged.union(&case_state);
                }
            }

            if let Some(default_case) = &match_expr.default_case {
                let mut default_state = state.clone();
                verify_expression(default_case, &mut default_state, fn_name);
                if first {
                    merged = default_state;
                } else {
                    merged.union(&default_state);
                }
            }

            *state = merged;
        }

        Expression::For(for_expr) => {
            verify_expression(&for_expr.start_range, state, fn_name);
            verify_expression(&for_expr.end_range, state, fn_name);

            let mut loop_state = state.clone();
            verify_expression(&for_expr.block, &mut loop_state, fn_name);
            verify_expression(&for_expr.block, &mut loop_state, fn_name);
            state.union(&loop_state);
        }

        Expression::Loop(body) => {
            let mut loop_state = state.clone();
            verify_expression(body, &mut loop_state, fn_name);
            verify_expression(body, &mut loop_state, fn_name);
            state.union(&loop_state);
        }

        Expression::While(while_expr) => {
            let mut loop_state = state.clone();
            verify_expression(&while_expr.condition, &mut loop_state, fn_name);
            verify_expression(&while_expr.body, &mut loop_state, fn_name);
            verify_expression(&while_expr.condition, &mut loop_state, fn_name);
            verify_expression(&while_expr.body, &mut loop_state, fn_name);
            state.union(&loop_state);
        }

        Expression::Literal(literal) => {
            verify_literal(literal, state, fn_name);
        }

        // Unary: reference/dereference operands are non-consuming
        Expression::Unary(unary) => {
            if matches!(unary.operator, UnaryOp::Reference { .. } | UnaryOp::Dereference { .. }) {
                verify_reference_expression(&unary.rhs, state, fn_name);
            } else {
                verify_expression(&unary.rhs, state, fn_name);
            }
        }

        Expression::Binary(binary) => {
            verify_expression(&binary.lhs, state, fn_name);
            verify_expression(&binary.rhs, state, fn_name);
        }

        // Index: collection is in reference position (not consumed),
        // index is a normal expression
        Expression::Index(index) => {
            verify_reference_expression(&index.collection, state, fn_name);
            verify_expression(&index.index, state, fn_name);
        }

        Expression::Cast(cast) => {
            verify_expression(&cast.lhs, state, fn_name);
        }

        Expression::Tuple(elements) => {
            for elem in elements {
                verify_expression(elem, state, fn_name);
            }
        }

        // ExtractTupleField: the ownership pass delays clones to the
        // outermost extract. The inner tuple/ident is in reference position.
        Expression::ExtractTupleField(tuple, _) => {
            if !verify_extract_rec(tuple, state, fn_name) {
                // Fallback: the inner expression is not an ident/extract/deref chain,
                // so process it as a normal (consuming) expression.
                verify_expression(tuple, state, fn_name);
            }
        }

        Expression::Call(call) => {
            verify_expression(&call.func, state, fn_name);
            for arg in &call.arguments {
                verify_expression(arg, state, fn_name);
            }
        }

        Expression::Constrain(boolean, _, msg) => {
            verify_expression(boolean, state, fn_name);
            if let Some(msg) = msg {
                verify_expression(&msg.0, state, fn_name);
            }
        }

        Expression::Semi(inner) => {
            verify_expression(inner, state, fn_name);
        }

        Expression::Break | Expression::Continue => (),
    }
}

/// Verify an expression that is in a *reference* position — its value is
/// borrowed, not consumed. Mirrors the ownership pass's
/// `handle_reference_expression`.
///
/// Idents in this context are NOT consumed. But we still verify they
/// haven't been previously moved.
fn verify_reference_expression(expr: &Expression, state: &mut MoveState, fn_name: &str) {
    match expr {
        Expression::Ident(ident) => {
            if let Definition::Local(local_id) = &ident.definition
                && contains_array_or_str_type(&ident.typ)
            {
                assert!(
                    !state.is_moved(*local_id),
                    "Use-after-move in function `{fn_name}`: variable `{}` (local {:?}) \
                     used in reference position after being consumed",
                    ident.name,
                    local_id,
                );
                // NOT marked as moved — reference position preserves ownership.
            }
        }

        // Clone in reference position: also non-consuming
        Expression::Clone(inner) => {
            verify_non_consuming(inner, state, fn_name);
        }

        // In `&{ a; b; ...; z }` only z is in reference position
        Expression::Block(exprs) => {
            let len_minus_one = exprs.len().saturating_sub(1);
            for expr in exprs.iter().take(len_minus_one) {
                verify_expression(expr, state, fn_name);
            }
            if let Some(expr) = exprs.last() {
                verify_reference_expression(expr, state, fn_name);
            }
        }

        // Dereference operand is also in reference position
        Expression::Unary(Unary { rhs, operator: UnaryOp::Dereference { .. }, .. }) => {
            verify_reference_expression(rhs, state, fn_name);
        }

        Expression::ExtractTupleField(tuple, _) => {
            verify_reference_expression(tuple, state, fn_name);
        }

        Expression::Index(index) => {
            verify_reference_expression(&index.collection, state, fn_name);
            verify_expression(&index.index, state, fn_name);
        }

        // Anything else (e.g. a function call) is treated normally
        other => verify_expression(other, state, fn_name),
    }
}

/// Walk the inner chain of an `ExtractTupleField` expression.
/// Returns true if the chain is all idents/extracts/derefs (handled as
/// non-consuming), false if we encounter something else that needs normal
/// processing.
///
/// Mirrors the ownership pass's `handle_extract_expression_rec`.
fn verify_extract_rec(expr: &Expression, state: &mut MoveState, fn_name: &str) -> bool {
    match expr {
        Expression::Ident(ident) => {
            if let Definition::Local(local_id) = &ident.definition
                && contains_array_or_str_type(&ident.typ)
            {
                assert!(
                    !state.is_moved(*local_id),
                    "Use-after-move in function `{fn_name}`: variable `{}` (local {:?}) \
                     used in tuple extraction after being consumed",
                    ident.name,
                    local_id,
                );
            }
            true
        }
        Expression::Unary(Unary { rhs, operator: UnaryOp::Dereference { .. }, .. }) => {
            verify_reference_expression(rhs, state, fn_name);
            true
        }
        Expression::ExtractTupleField(tuple, _) => verify_extract_rec(tuple, state, fn_name),
        _ => false,
    }
}

/// Verify an expression that is known to be non-consuming (inside Clone/Drop).
/// Checks that referenced idents haven't been moved, but does NOT mark
/// anything as moved.
fn verify_non_consuming(expr: &Expression, state: &mut MoveState, fn_name: &str) {
    match expr {
        Expression::Ident(ident) => {
            if let Definition::Local(local_id) = &ident.definition
                && contains_array_or_str_type(&ident.typ)
            {
                assert!(
                    !state.is_moved(*local_id),
                    "Use-after-move in function `{fn_name}`: variable `{}` (local {:?}) \
                     used in clone/drop after being consumed",
                    ident.name,
                    local_id,
                );
            }
        }
        // For ExtractTupleField inside Clone: e.g. `tuple.0.clone()`
        // The tuple ident is non-consuming.
        Expression::ExtractTupleField(tuple, _) => {
            verify_non_consuming(tuple, state, fn_name);
        }
        // For any other expression inside Clone, recurse normally.
        // E.g. `foo()[0].clone()` — the call and index are normal expressions.
        other => verify_expression(other, state, fn_name),
    }
}

fn verify_literal(literal: &Literal, state: &mut MoveState, fn_name: &str) {
    match literal {
        Literal::Integer(..) | Literal::Bool(_) | Literal::Unit | Literal::Str(_) => (),
        Literal::FmtStr(_, _, captures) => verify_expression(captures, state, fn_name),
        Literal::Array(array) | Literal::Vector(array) => {
            for element in &array.contents {
                verify_expression(element, state, fn_name);
            }
        }
        Literal::Repeated { element, .. } => verify_expression(element, state, fn_name),
    }
}

fn verify_lvalue(lvalue: &LValue, state: &mut MoveState, fn_name: &str) {
    match lvalue {
        LValue::Ident(_) => (),
        LValue::Index { array, index, .. } => {
            verify_expression(index, state, fn_name);
            verify_lvalue(array, state, fn_name);
        }
        LValue::MemberAccess { object, .. } => {
            verify_lvalue(object, state, fn_name);
        }
        LValue::Dereference { reference, .. } => {
            verify_lvalue(reference, state, fn_name);
        }
        LValue::Clone(inner) => {
            verify_lvalue(inner, state, fn_name);
        }
    }
}
