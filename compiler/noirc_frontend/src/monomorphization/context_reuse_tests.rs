//! Tests that monomorphizing an entry point gives the same program whether or not other entry
//! points were monomorphized against the same elaborated context first.
//!
//! A `NodeInterner` is reusable across entry points — `nargo export` monomorphizes every exported
//! function of one context, and `nargo test` reuses one context across the tests of a package —
//! so what an entry point compiles to must not depend on what was compiled before it.
#![cfg(test)]

use crate::hir::Context;
use crate::hir::FunctionNameMatch;
use crate::monomorphization::monomorphize;
use crate::node_interner::FuncId;
use crate::test_utils::get_program;

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
    match monomorphize(function, &context.def_interner, false) {
        Ok(program) => program.to_string(),
        Err(error) => format!("{error:?}"),
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

/// `intermediate_underflow::<0>` fails converting a type in its own body, so monomorphization
/// stops part way through a generic function's job, with `N` at the call site's `0`.
#[test]
fn failing_to_monomorphize_a_generic_function_is_order_independent() {
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
    assert_monomorphization_is_order_independent(src);
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
    assert_monomorphization_is_order_independent(src);
}

/// Both tests reach one trait-method call site whose receiver resolves to a different impl in
/// each, so the call site's instantiation bindings are extended with a different impl's in each.
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
    assert_monomorphization_is_order_independent(src);
}

/// A trait with an associated constant, used through two impls.
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
    assert_monomorphization_is_order_independent(src);
}

/// A trait method reached through a `where` clause on a generic struct's method, so the impl is
/// only settled once monomorphization searches for it, and that search's bindings apply only while
/// the impl's method is compiled.
#[test]
fn resolving_an_assumed_impl_is_order_independent() {
    let src = r#"
        trait MyHasher {
            fn finish_hash(self) -> Field;
        }

        trait MyDefault {
            fn my_default() -> Self;
        }

        trait BuildMyHasher<H> where H: MyHasher {
            fn build(self) -> H;
        }

        struct DefaultBuilder<H> {}

        impl<H> BuildMyHasher<H> for DefaultBuilder<H> where H: MyHasher + MyDefault {
            fn build(self) -> H {
                H::my_default()
            }
        }

        struct Holder<B> {
            builder: B,
        }

        impl<B> Holder<B> {
            fn compute<H>(self) -> Field where B: BuildMyHasher<H>, H: MyHasher {
                self.builder.build().finish_hash()
            }
        }

        struct Concrete {}

        impl MyDefault for Concrete {
            fn my_default() -> Self { Concrete {} }
        }

        impl MyHasher for Concrete {
            fn finish_hash(self) -> Field { 7 }
        }

        #[test]
        fn hashes_through_a_where_clause() {
            let holder = Holder { builder: DefaultBuilder::<Concrete> {} };
            assert_eq(holder.compute(), 7);
        }

        #[test]
        fn hashes_through_a_where_clause_again() {
            let holder = Holder { builder: DefaultBuilder::<Concrete> {} };
            assert_eq(holder.compute(), 7);
        }
    "#;
    assert_monomorphization_is_order_independent(src);
}
