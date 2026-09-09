//! Tests for the invariant that monomorphizing an entry point leaves the elaborated context
//! exactly as it found it.
//!
//! A `NodeInterner` is reusable across entry points — `nargo export` monomorphizes every exported
//! function of one context, and `nargo test` reuses one context across the tests of a package.
//! That is only sound while monomorphization is pure: a type variable left bound, or a call
//! site's instantiation bindings left overwritten, is visible to every later compilation against
//! the same context, and can silently change what it produces.
#![cfg(test)]

use std::collections::BTreeMap;

use crate::TypeBinding;
use crate::hir::Context;
use crate::hir::FunctionNameMatch;
use crate::monomorphization::monomorphize;
use crate::node_interner::{FuncId, NodeInterner};
use crate::test_utils::get_program;

/// The binding state of every type variable the interner has stored instantiation bindings for,
/// rendered as text so a mismatch reports which variable changed and what it changed to.
///
/// These are the variables monomorphization binds in order to resolve a call site's generics, so
/// they are the ones at risk of being left bound. Keyed by variable id, which is unique per
/// variable and stable across the calls being compared.
fn binding_snapshot(interner: &NodeInterner) -> BTreeMap<usize, String> {
    let mut snapshot = BTreeMap::new();
    for (_, bindings) in interner.all_instantiation_bindings() {
        for (var, _kind, _binding) in bindings.values() {
            let state = match &*var.borrow() {
                TypeBinding::Bound(typ) => format!("bound to {typ}"),
                TypeBinding::Unbound(..) => "unbound".to_string(),
            };
            snapshot.insert(var.id().0, state);
        }
    }
    snapshot
}

/// Every `#[test]` function in the root crate, ordered by name so that the shared-context run
/// and the fresh-context runs visit them in the same order.
fn test_functions(context: &Context) -> Vec<(String, FuncId)> {
    let crate_id = context.root_crate_id();
    let mut functions: Vec<_> = context
        .get_all_test_functions_in_crate_matching(crate_id, &FunctionNameMatch::Anything)
        .into_iter()
        .map(|(name, function)| (name, function.id))
        .collect();
    functions.sort_by(|(left, _), (right, _)| left.cmp(right));
    functions
}

/// Monomorphize `function` against `context`, returning the program it produced or the error it
/// failed with, both rendered as text so the two can be compared and displayed.
fn monomorphize_to_string(context: &mut Context, function: FuncId) -> String {
    let files = context.file_manager.as_file_map();
    match monomorphize(function, &mut context.def_interner, files, false) {
        Ok(program) => program.to_string(),
        Err(error) => format!("{error:?}"),
    }
}

/// Assert that monomorphizing each `#[test]` function in `src` leaves the type variables the
/// context holds instantiation bindings for exactly as they were before.
///
/// This is the invariant directly: a variable left bound is a difference in the context that a
/// later compilation can read.
fn assert_monomorphization_restores_bindings(src: &str) {
    let (_, mut context, _) = get_program(src);

    for (name, function) in test_functions(&context) {
        let before = binding_snapshot(&context.def_interner);
        let _ = monomorphize_to_string(&mut context, function);
        let after = binding_snapshot(&context.def_interner);

        assert_eq!(
            before, after,
            "monomorphizing `{name}` left type variables in the context bound differently to how \
             it found them, so a later compilation against this context can see the bindings it \
             applied"
        );
    }
}

/// Assert that each `#[test]` function in `src` monomorphizes to the same thing whether it is
/// compiled against a context that has already compiled the tests before it, or against a
/// context of its own.
///
/// This is the property a test runner depends on when it shares a context: a test's result must
/// not depend on what ran before it.
fn assert_monomorphization_is_order_independent(src: &str) {
    let names: Vec<String> = {
        let (_, context, _) = get_program(src);
        let functions = test_functions(&context);
        assert!(!functions.is_empty(), "test source contains no `#[test]` functions");
        functions.into_iter().map(|(name, _)| name).collect()
    };

    // What each entry point compiles to against a context nothing else has touched.
    let alone: Vec<String> = names
        .iter()
        .map(|name| {
            let (_, mut context, _) = get_program(src);
            let (_, function) = test_functions(&context)
                .into_iter()
                .find(|(candidate, _)| candidate == name)
                .expect("elaborating the same source twice yields the same test functions");
            monomorphize_to_string(&mut context, function)
        })
        .collect();

    // What they compile to against one context, in order.
    let (_, mut context, _) = get_program(src);
    for ((name, function), expected) in test_functions(&context).into_iter().zip(alone) {
        let shared = monomorphize_to_string(&mut context, function);
        assert_eq!(
            shared, expected,
            "monomorphizing `{name}` against a context that had already compiled the entry \
             points before it produced a different result to compiling it against a fresh context"
        );
    }
}

fn assert_monomorphization_is_pure(src: &str) {
    assert_monomorphization_restores_bindings(src);
    assert_monomorphization_is_order_independent(src);
}

/// `intermediate_underflow::<0>` fails converting a type in its own body, which happens while
/// the queued job that monomorphizes it holds `N` bound to the call site's `0`. This is the shape
/// of failure that leaves a generic bound: the error travels out of the job that applied the
/// bindings, past the point where they would have been undone.
#[test]
fn failing_to_monomorphize_a_generic_function_restores_its_bindings() {
    let src = r#"
        fn intermediate_underflow<let N: u32>() -> Field {
            let result: [Field; (N - 1) + 1] = [0; (N - 1) + 1];
            result[0]
        }

        #[test]
        fn fails_inside_a_generic_function() {
            let _ = intermediate_underflow::<0>();
        }

        #[test]
        fn succeeds_in_the_same_generic_function() {
            let _ = intermediate_underflow::<5>();
        }
    "#;
    assert_monomorphization_is_pure(src);
}

/// Both tests reach the same generic function at the same call site, at different instantiations.
#[test]
fn one_generic_call_site_at_two_instantiations_is_order_independent() {
    let src = r#"
        fn first<T>(items: [T; 2]) -> T {
            items[0]
        }

        fn first_of_pair<T>(a: T, b: T) -> T {
            first([a, b])
        }

        #[test]
        fn instantiates_at_field() {
            assert_eq(first_of_pair(1, 2), 1);
        }

        #[test]
        fn instantiates_at_u32() {
            assert_eq(first_of_pair(1 as u32, 2 as u32), 1 as u32);
        }
    "#;
    assert_monomorphization_is_pure(src);
}

/// Both tests reach one trait-method call site whose receiver resolves to a different impl in
/// each. Resolving the impl rewrites the call site's stored instantiation bindings, so this is
/// the case where a context carries a difference forward on the success path.
#[test]
fn one_trait_method_call_site_at_two_impls_is_order_independent() {
    let src = r#"
        trait Doubles {
            fn double(self) -> Self;
        }

        impl Doubles for Field {
            fn double(self) -> Field { self + self }
        }

        impl Doubles for u32 {
            fn double(self) -> u32 { self + self }
        }

        fn double_twice<T>(x: T) -> T where T: Doubles {
            x.double().double()
        }

        #[test]
        fn doubles_a_field() {
            assert_eq(double_twice(1), 4);
        }

        #[test]
        fn doubles_a_u32() {
            assert_eq(double_twice(1 as u32), 4 as u32);
        }
    "#;
    assert_monomorphization_is_pure(src);
}

/// A trait with an associated constant, used through two impls. Resolving the constant returns
/// from the middle of trait item resolution, after the call site's bindings have been rewritten.
#[test]
fn a_trait_associated_constant_at_two_impls_is_order_independent() {
    let src = r#"
        trait Sized2 {
            let N: u32;
        }

        impl Sized2 for Field {
            let N: u32 = 1;
        }

        impl Sized2 for u32 {
            let N: u32 = 2;
        }

        fn size_of<T>(_x: T) -> u32 where T: Sized2 {
            <T as Sized2>::N
        }

        #[test]
        fn size_of_a_field() {
            assert_eq(size_of(1), 1);
        }

        #[test]
        fn size_of_a_u32() {
            assert_eq(size_of(1 as u32), 2);
        }
    "#;
    assert_monomorphization_is_pure(src);
}
