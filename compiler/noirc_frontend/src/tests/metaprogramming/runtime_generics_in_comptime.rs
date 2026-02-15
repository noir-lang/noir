//! Tests for detecting runtime generics used in comptime contexts.
//!
//! A comptime block inside a non-comptime function runs during elaboration, before
//! monomorphization resolves generics. These tests verify that we report errors when
//! runtime generics leak into comptime blocks.

use crate::test_utils::stdlib_src;
use crate::tests::{assert_no_errors, check_errors, check_errors_with_stdlib};

#[test]
fn error_on_generic_trait_method_in_comptime_block() {
    let src = r#"
    trait B {
        comptime fn c() -> u8;
    }

    pub fn g<F: B>() {
        let _c = comptime {
            F::c()
            ^^^^ Type `F` contains a generic from a runtime function and cannot be used in a comptime context
            ~~~~ This comptime block references a generic from an enclosing runtime function which will not be resolved until monomorphization, which runs after comptime evaluation
        };
    }

    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn error_on_generic_as_trait_path_in_comptime_block() {
    let src = r#"
    trait B {
        comptime fn c() -> u8;
    }

    pub fn g<F: B>() {
        let _c = comptime {
            <F as B>::c()
             ^^^^^^ Type `F` contains a generic from a runtime function and cannot be used in a comptime context
             ~~~~~~ This comptime block references a generic from an enclosing runtime function which will not be resolved until monomorphization, which runs after comptime evaluation
        };
    }

    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn no_error_on_concrete_trait_method_in_comptime_block() {
    let src = r#"
    trait B {
        comptime fn c() -> u8;
    }

    struct Foo {}

    impl B for Foo {
        comptime fn c() -> u8 { 42 }
    }

    pub fn main() {
        let _c = comptime { Foo::c() };
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn no_error_on_generic_trait_method_in_comptime_function() {
    // Comptime functions are monomorphized by the interpreter at call time,
    // so their generics are resolved before trait dispatch happens.
    let src = r#"
    trait B {
        comptime fn c() -> u8;
    }

    struct Foo {}

    impl B for Foo {
        comptime fn c() -> u8 { 42 }
    }

    comptime fn call_c<F: B>() -> u8 {
        F::c()
    }

    pub fn main() {
        let _c = comptime { call_c::<Foo>() };
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn error_on_runtime_generic_type_annotation_in_comptime_block() {
    let src = r#"
    pub fn g<F>() {
        comptime {
            let _x: [F; 1] = zeroed();
                    ^^^^^^ Type `[F; 1]` contains a generic from a runtime function and cannot be used in a comptime context
                    ~~~~~~ This comptime block references a generic from an enclosing runtime function which will not be resolved until monomorphization, which runs after comptime evaluation
        };
    }

    fn main() {}
    "#;
    check_errors_with_stdlib(src, [stdlib_src::ZEROED]);
}

#[test]
fn error_on_runtime_generic_type_annotation_with_comptime_rhs() {
    let src = r#"
    pub fn g<F>() {
        let _: F = comptime { zeroed() };
                   ^^^^^^^^^^^^^^^^^^^^^ Type `F` contains a generic from a runtime function and cannot be used in a comptime context
                   ~~~~~~~~~~~~~~~~~~~~~ This comptime block references a generic from an enclosing runtime function which will not be resolved until monomorphization, which runs after comptime evaluation
                   ^^^^^^^^^^^^^^^^^^^^^ Cannot inline values of type `F` into this position
                   ~~~~~~~~~~~~~~~~~~~~~ Cannot inline value `(zeroed F)`
    }

    fn main() {}
    "#;
    check_errors_with_stdlib(src, [stdlib_src::ZEROED]);
}

#[test]
fn error_on_runtime_generic_comptime_block_passed_to_generic_function() {
    let src = r#"
    fn id<T>(x: T) -> T { x }

    pub fn g<F>() {
        let _ = id(comptime { zeroed::<F>() });
                   ^^^^^^^^^^^^^^^^^^^^^^^^^^ Type `F` contains a generic from a runtime function and cannot be used in a comptime context
                   ~~~~~~~~~~~~~~~~~~~~~~~~~~ This comptime block references a generic from an enclosing runtime function which will not be resolved until monomorphization, which runs after comptime evaluation
                   ^^^^^^^^^^^^^^^^^^^^^^^^^^ Cannot inline values of type `F` into this position
                   ~~~~~~~~~~~~~~~~~~~~~~~~~~ Cannot inline value `(zeroed F)`
    }

    fn main() {}
    "#;
    check_errors_with_stdlib(src, [stdlib_src::ZEROED]);
}
