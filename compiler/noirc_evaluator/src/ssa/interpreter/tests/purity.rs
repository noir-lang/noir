//! Tests for the interpreter's purity-contract checks: a function whose recorded
//! [Purity] is `Pure` or `PureWithPredicate` must behave accordingly during
//! interpretation, or the interpreter reports a [InterpreterError::PurityViolation].
//!
//! Violations can only arise from a bug in purity analysis (or in a pass that
//! invalidates its results), so these tests inject hand-crafted purities instead of
//! running the real analysis, which would classify the functions correctly.

use std::sync::Arc;

use crate::ssa::{
    interpreter::{IResults, errors::InterpreterError, intrinsics::check_intrinsic_mutation_label},
    ir::{function::FunctionId, instruction::Intrinsic},
    opt::pure::{FunctionPurities, Purity},
};

use super::{Ssa, Value};

/// Interpret `src` with explicitly chosen purities instead of the computed ones,
/// simulating a buggy or stale purity analysis.
fn interpret_with_injected_purities(
    src: &str,
    purities: &[(u32, Purity)],
    args: Vec<Value>,
) -> IResults {
    let mut ssa = Ssa::from_str(src).unwrap();

    let mut map = FunctionPurities::default();
    for (id, purity) in purities {
        map.insert_purity(FunctionId::new(*id), *purity);
    }
    for function in ssa.functions.values() {
        if function.runtime().is_brillig() {
            map.insert_brillig_function(function.id());
        }
    }

    let map = Arc::new(map);
    for function in ssa.functions.values_mut() {
        function.dfg.set_function_purities(map.clone());
    }
    ssa.interpret(args)
}

#[track_caller]
fn expect_purity_violation(result: IResults, reason_fragment: &str) {
    match result {
        Err(InterpreterError::PurityViolation { reason, .. }) => {
            assert!(
                reason.contains(reason_fragment),
                "expected purity violation reason to contain {reason_fragment:?}, got {reason:?}"
            );
        }
        other => panic!("expected a purity violation, got {other:?}"),
    }
}

#[test]
fn pure_function_that_fails_is_a_purity_violation() {
    // f1 is really `PureWithPredicate` (it contains a constrain); marking it `Pure`
    // simulates an analysis bug. A `Pure` function must never fail: DIE may remove
    // an unused call to it, which would erase the failure.
    let src = r#"
        acir(inline) fn main f0 {
          b0():
            call f1(u1 0)
            return
        }
        acir(inline) fn assert_true f1 {
          b0(v0: u1):
            constrain v0 == u1 1
            return
        }
    "#;
    let result = interpret_with_injected_purities(src, &[(1, Purity::Pure)], Vec::new());
    expect_purity_violation(result, "failed");
}

#[test]
fn predicate_pure_function_that_fails_is_not_a_purity_violation() {
    // `PureWithPredicate` functions are allowed to fail; the failure must surface
    // unchanged.
    let src = r#"
        acir(inline) fn main f0 {
          b0():
            call f1(u1 0)
            return
        }
        acir(inline) fn assert_true f1 {
          b0(v0: u1):
            constrain v0 == u1 1
            return
        }
    "#;
    let result =
        interpret_with_injected_purities(src, &[(1, Purity::PureWithPredicate)], Vec::new());
    assert!(
        matches!(result, Err(InterpreterError::ConstrainEqFailed { .. })),
        "expected the plain constrain failure, got {result:?}"
    );
}

#[test]
fn pure_function_mutating_argument_array_in_place_is_a_purity_violation() {
    // In Brillig an `array_set` on an array whose reference count is 1 mutates the
    // backing store in place, so the caller's copy observes the write. A function
    // doing this to one of its parameters cannot be `Pure` (or `PureWithPredicate`).
    let src = r#"
        brillig(inline) fn main f0 {
          b0():
            v2 = make_array [Field 1, Field 2] : [Field; 2]
            v4 = call f1(v2) -> [Field; 2]
            return
        }
        brillig(inline) fn set_first f1 {
          b0(v0: [Field; 2]):
            v3 = array_set mut v0, index u32 0, value Field 5
            return v3
        }
    "#;
    let result = interpret_with_injected_purities(src, &[(1, Purity::Pure)], Vec::new());
    expect_purity_violation(result, "mutated");
}

#[test]
fn predicate_pure_function_mutating_argument_vector_in_place_is_a_purity_violation() {
    // Vector-mutator intrinsics write through the input vector's backing store when
    // its reference count is 1. Mutation of caller-visible memory is forbidden for
    // `PureWithPredicate` functions too, not just `Pure` ones.
    let src = r#"
        brillig(inline) fn main f0 {
          b0():
            v3 = make_array [Field 1, Field 2] : [Field]
            v5, v6 = call f1(u32 2, v3) -> (u32, [Field])
            return
        }
        brillig(inline) fn push f1 {
          b0(v0: u32, v1: [Field]):
            v4, v5 = call vector_push_back(v0, v1, Field 3) -> (u32, [Field])
            return v4, v5
        }
    "#;
    let result =
        interpret_with_injected_purities(src, &[(1, Purity::PureWithPredicate)], Vec::new());
    expect_purity_violation(result, "mutated");
}

#[test]
fn pure_function_storing_through_argument_reference_is_a_purity_violation() {
    // Functions taking references are always `Impure`; marking one `Pure` simulates
    // an analysis bug. A store through a caller-provided reference is caller-visible.
    let src = r#"
        brillig(inline) fn main f0 {
          b0():
            v1 = allocate -> &mut Field
            store Field 1 at v1
            call f1(v1)
            return
        }
        brillig(inline) fn set_ref f1 {
          b0(v0: &mut Field):
            store Field 2 at v0
            return
        }
    "#;
    let result = interpret_with_injected_purities(src, &[(1, Purity::Pure)], Vec::new());
    expect_purity_violation(result, "mutated");
}

#[test]
fn pure_function_calling_foreign_function_is_a_purity_violation() {
    // Even a `#[pure]` oracle makes its caller at most `PureWithPredicate`, so a
    // foreign call inside a `Pure` function is always a contract violation.
    let src = r#"
        brillig(inline) fn main f0 {
          b0():
            call f1(Field 7)
            return
        }
        brillig(inline) fn print_it f1 {
          b0(v0: Field):
            v3 = make_array b"{\"kind\":\"field\"}"
            call print(u1 1, v0, v3, u1 0)
            return
        }
    "#;
    let result = interpret_with_injected_purities(src, &[(1, Purity::Pure)], Vec::new());
    expect_purity_violation(result, "foreign function");
}

#[test]
fn pure_function_may_mutate_local_memory() {
    // Local allocations and local arrays are invisible to the caller: mutating them
    // is allowed in a `Pure` function even though the same instructions on a
    // parameter would be a violation.
    let src = r#"
        brillig(inline) fn main f0 {
          b0():
            v1 = call f1() -> Field
            return v1
        }
        brillig(inline) fn local_mutations f1 {
          b0():
            v1 = allocate -> &mut Field
            store Field 1 at v1
            store Field 2 at v1
            v4 = make_array [Field 1, Field 2] : [Field; 2]
            v7 = array_set mut v4, index u32 0, value Field 5
            v9 = load v1 -> Field
            return v9
        }
    "#;
    let result = interpret_with_injected_purities(src, &[(1, Purity::Pure)], Vec::new());
    assert!(result.is_ok(), "expected success, got {result:?}");
}

#[test]
fn acir_in_place_array_set_on_a_parameter_is_not_a_purity_violation() {
    // In a constrained context arrays have value semantics: the Mutable Array Set
    // Optimizations pass marks an `array_set` mutable when the old array value is
    // dead, so the in-place write only reuses that value's backing store. It is not
    // a caller-visible mutation, and the function legitimately stays
    // `predicate_pure` under the real analysis.
    let src = r#"
        acir(inline) fn main f0 {
          b0():
            v3 = make_array [Field 1, Field 2] : [Field; 2]
            v5 = call f1(v3) -> Field
            constrain v5 == Field 5
            return
        }
        acir(inline) fn set_first f1 {
          b0(v0: [Field; 2]):
            v4 = array_set mut v0, index u32 0, value Field 5
            v6 = array_get v4, index u32 0 -> Field
            return v6
        }
    "#;
    let ssa = Ssa::from_str(src).unwrap().purity_analysis();
    let result = ssa.interpret(Vec::new());
    assert!(result.is_ok(), "expected success, got {result:?}");
}

#[test]
fn in_place_vector_mutation_requires_the_mutator_label() {
    // Every intrinsic the interpreter mutates a vector through must declare itself in
    // `Intrinsic::mutates_array_operand_in_brillig`: purity analysis classifies the
    // containing function from that list, so an unlisted mutator would silently
    // poison recorded purities. The six vector mutators pass; a non-mutator is
    // rejected at the moment of mutation.
    let mutators = [
        Intrinsic::VectorPushBack,
        Intrinsic::VectorPushFront,
        Intrinsic::VectorPopBack,
        Intrinsic::VectorPopFront,
        Intrinsic::VectorInsert,
        Intrinsic::VectorRemove,
    ];
    for intrinsic in mutators {
        assert!(check_intrinsic_mutation_label(intrinsic).is_ok());
    }

    let result = check_intrinsic_mutation_label(Intrinsic::ArrayLen);
    assert!(
        matches!(
            result,
            Err(InterpreterError::IntrinsicPurityViolation { intrinsic: Intrinsic::ArrayLen })
        ),
        "expected an intrinsic purity violation, got {result:?}"
    );
}

#[test]
fn computed_purities_hold_on_a_pure_call_chain() {
    // End-to-end sanity: run the real purity analysis and interpret; correctly
    // classified functions must not trip the checks.
    let src = r#"
        acir(inline) fn main f0 {
          b0(v0: Field):
            v2 = call f1(v0) -> Field
            constrain v2 == Field 6
            return
        }
        acir(inline) fn double f1 {
          b0(v0: Field):
            v1 = add v0, v0
            return v1
        }
    "#;
    let ssa = Ssa::from_str(src).unwrap().purity_analysis();
    let result = ssa.interpret(vec![Value::field(3_u32.into())]);
    assert!(result.is_ok(), "expected success, got {result:?}");
}
