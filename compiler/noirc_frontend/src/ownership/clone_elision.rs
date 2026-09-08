//! Decides when the `Clone` the ownership pass would otherwise insert around a call
//! argument can be skipped ("clone elision").
//!
//! Foreign calls (oracles) only read their inputs — values are copied across the
//! runtime boundary — and many builtins neither modify their array arguments nor
//! return an alias of them. An array passed by value to such a callee needs no
//! defensive reference count bump even when the argument is not at its last use.
//! The same property propagates through thin wrappers that only forward their
//! arguments to a foreign call (e.g. `println` -> `print_unconstrained` -> `print`
//! oracle).
//!
//! Even for these callees, eliding the clone on one argument is only sound if
//! nothing that executes between materializing that argument and issuing the call
//! can write into the buffer the argument names. Arguments are evaluated left to
//! right, so a later argument such as `bump(&mut x, i)` runs while an earlier,
//! already-materialized `x` waits for the call. Without the clone the buffer's
//! reference count is 1 and Brillig's copy-on-write mutates it in place, so the
//! callee would read the mutated buffer. The clone on an argument is therefore
//! kept unless every later argument of the same call is side-effect-free.

use rustc_hash::FxHashSet as HashSet;

use crate::monomorphization::ast::{Call, Definition, Expression, FuncId, Literal, Program};
use crate::shared::Builtin;

/// Return whether a call to the given builtin or low level function allows the
/// `Clone` around its array arguments to be elided: the callee must neither modify
/// the input nor return an alias of it that a later Brillig mutation could observe.
///
/// This mirrors `Intrinsic::purity` and `Intrinsic::unsafe_for_clone_elision_in_brillig`
/// in `noirc_evaluator`, which cannot be used here directly since `noirc_frontend` cannot
/// depend on `noirc_evaluator`. A test in `noirc_evaluator` (`ssa::ssa_gen::tests`) checks
/// that this function agrees with them for every builtin.
pub fn builtin_supports_clone_elision(builtin: Builtin) -> bool {
    match builtin {
        // Pure or predicate-pure builtins that neither modify their array arguments
        // nor return an alias of them.
        Builtin::ArrayLen
        | Builtin::AsVector
        | Builtin::AssertConstant
        | Builtin::StaticAssert
        | Builtin::ApplyRangeConstraint
        | Builtin::ToLeRadix
        | Builtin::ToBeRadix
        | Builtin::ToLeBits
        | Builtin::ToBeBits
        | Builtin::AsWitness
        | Builtin::IsUnconstrained
        | Builtin::DerivePedersenGenerators
        | Builtin::FieldLessThan => true,

        // Vector mutators may write through their input pointer in place when the
        // copy-on-write reference count is 1. `str_as_bytes` and
        // `array_as_str_unchecked` are no-op conversions whose result aliases their
        // input, so a later mutation of the result could corrupt the source.
        // `black_box` is deliberately opaque to the optimizer, and reference count
        // reads are ordering-dependent on the rc traffic around them.
        Builtin::VectorPushBack
        | Builtin::VectorPushFront
        | Builtin::VectorPopBack
        | Builtin::VectorPopFront
        | Builtin::VectorInsert
        | Builtin::VectorRemove
        | Builtin::StrAsBytes
        | Builtin::ArrayAsStrUnchecked
        | Builtin::BlackBoxHint
        | Builtin::ArrayRefcount
        | Builtin::VectorRefcount => false,

        // Black box functions (hashes, curve operations, signature verification)
        // only read their inputs and return fresh outputs.
        Builtin::BlackBox(_) => true,

        // Anything else is either evaluated away before SSA generation
        // (comptime-only and monomorphizer-handled builtins) or unknown;
        // conservatively keep the clone.
        _ => false,
    }
}

/// Find every function whose body, after peeling block/semi wrapping, is exactly one
/// [`Call`] whose target is either an oracle directly or another oracle wrapper, and
/// whose arguments are structurally side-effect-free.
///
/// Such "thin wrappers" inherit the input-preserving property of oracles: foreign
/// calls only read their inputs, so a wrapper that forwards to one cannot modify its
/// array arguments either.
pub fn find_oracle_wrappers(program: &Program) -> HashSet<FuncId> {
    /// Maximum recursion depth for wrapper chains. Real chains are 2–3 deep
    /// (e.g. `println` -> `print_unconstrained` -> `print` oracle); the bound only
    /// exists to keep pathological inputs from blowing the stack.
    const ORACLE_WRAPPER_MAX_DEPTH: u32 = 5;

    /// `depth` is the maximum remaining recursion depth; reaching zero bails out conservatively.
    fn is_oracle_wrapper(func_id: FuncId, program: &Program, depth: u32) -> bool {
        if depth == 0 {
            return false;
        }
        let Some(inner) = peel_to_single_call(&program[func_id].body) else {
            return false;
        };
        if !inner.arguments.iter().all(is_side_effect_free) {
            return false;
        }
        match callee_definition(&inner.func) {
            Some(Definition::Oracle { .. }) => true,
            Some(Definition::Function(inner_id)) => {
                is_oracle_wrapper(*inner_id, program, depth - 1)
            }
            _ => false,
        }
    }

    program
        .functions
        .iter()
        .filter(|function| is_oracle_wrapper(function.id, program, ORACLE_WRAPPER_MAX_DEPTH))
        .map(|function| function.id)
        .collect()
}

/// Return whether the callee of a call is known not to modify its array arguments
/// nor return an alias of them, so that the `Clone` around each argument may be
/// elided (sibling arguments permitting, see [`elide_clones_in_call_arguments`]).
fn callee_preserves_array_arguments(func: &Expression, oracle_wrappers: &HashSet<FuncId>) -> bool {
    match callee_definition(func) {
        Some(Definition::Builtin(builtin) | Definition::LowLevel(builtin)) => {
            builtin_supports_clone_elision(*builtin)
        }
        Some(Definition::Oracle { .. }) => true,
        Some(Definition::Function(func_id)) => oracle_wrappers.contains(func_id),
        _ => false,
    }
}

fn callee_definition(func: &Expression) -> Option<&Definition> {
    match func {
        Expression::Ident(ident) => Some(&ident.definition),
        _ => None,
    }
}

/// Remove the top-level `Clone` from each argument of `call` for which elision is
/// sound: the callee must preserve its arguments, and every argument evaluated
/// *after* the cloned one must be side-effect-free, so that nothing can write into
/// the buffer between its materialization and the call itself.
pub(super) fn elide_clones_in_call_arguments(call: &mut Call, oracle_wrappers: &HashSet<FuncId>) {
    if !callee_preserves_array_arguments(&call.func, oracle_wrappers) {
        return;
    }

    let mut later_args_side_effect_free = true;
    for argument in call.arguments.iter_mut().rev() {
        if later_args_side_effect_free && let Expression::Clone(inner) = argument {
            *argument = std::mem::replace(inner.as_mut(), Expression::Literal(Literal::Unit));
        }
        later_args_side_effect_free = later_args_side_effect_free && is_side_effect_free(argument);
    }
}

/// If `expr` is a block or `Semi` wrapping that ultimately reduces to a single
/// [`Call`], return that call. Otherwise return `None`.
fn peel_to_single_call(expr: &Expression) -> Option<&Call> {
    match expr {
        Expression::Call(call) => Some(call),
        Expression::Semi(inner) => peel_to_single_call(inner),
        Expression::Block(stmts) if stmts.len() == 1 => peel_to_single_call(&stmts[0]),
        _ => None,
    }
}

/// Conservatively check whether evaluating `expr` cannot mutate any caller-visible state.
///
/// Anything not on this whitelist — `Block`, `Semi`, `Assign`, `Let`, nested `Call`,
/// control flow, etc. — is treated as potentially side-effectful.
fn is_side_effect_free(expr: &Expression) -> bool {
    match expr {
        Expression::Ident(_) => true,
        Expression::Literal(lit) => match lit {
            Literal::Array(arr) | Literal::Vector(arr) => {
                arr.contents.iter().all(is_side_effect_free)
            }
            Literal::Repeated { element, .. } => is_side_effect_free(element),
            Literal::Integer(..) | Literal::Bool(_) | Literal::Unit | Literal::Str(_) => true,
            Literal::FmtStr(_, _, inner) => is_side_effect_free(inner),
        },
        Expression::ExtractTupleField(inner, _) => is_side_effect_free(inner),
        Expression::Tuple(items) => items.iter().all(is_side_effect_free),
        Expression::Index(idx) => {
            is_side_effect_free(&idx.collection) && is_side_effect_free(&idx.index)
        }
        Expression::Cast(cast) => is_side_effect_free(&cast.lhs),
        Expression::Unary(u) => is_side_effect_free(&u.rhs),
        Expression::Binary(b) => is_side_effect_free(&b.lhs) && is_side_effect_free(&b.rhs),
        Expression::Clone(inner) => is_side_effect_free(inner),
        _ => false,
    }
}
