//! Noir supports multiple runtime environments. This module contains tests related to runtime boundaries and entry point creation.
//! "Runtime boundaries" can refer to calls across the unconstrained/constrained boundary, valid attributes in vanilla programs vs. contracts, defining program entry points, etc.

use crate::tests::{assert_no_errors, check_errors};

#[test]
fn cannot_call_unconstrained_function_outside_of_unsafe() {
    let src = r#"
    fn main() {
        foo();
        ^^^^^ Call to unconstrained function is unsafe and must be in an unconstrained function or unsafe block
    }

    unconstrained fn foo() {}
    "#;
    check_errors(src);
}

#[test]
fn cannot_call_unconstrained_first_class_function_outside_of_unsafe() {
    let src = r#"
    fn main() {
        let func = foo;
        func();
        ^^^^^^ Call to unconstrained function is unsafe and must be in an unconstrained function or unsafe block
        inner(func);
    }

    fn inner(x: unconstrained fn() -> ()) {
        x();
        ^^^ Call to unconstrained function is unsafe and must be in an unconstrained function or unsafe block
    }

    unconstrained fn foo() {}
    "#;
    check_errors(src);
}

#[test]
fn missing_unsafe_block_when_needing_type_annotations() {
    // This test is a regression check that even when an unsafe block is missing
    // that we still appropriately continue type checking and infer type annotations.
    let src = r#"
    fn main() {
        let z = BigNum { limbs: [2, 0, 0] };
        assert(z.__is_zero() == false);
    }

    struct BigNum<let N: u32> {
        limbs: [u64; N],
    }

    impl<let N: u32> BigNum<N> {
        unconstrained fn __is_zero_impl(self) -> bool {
            let mut result: bool = true;
            for i in 0..N {
                result = result & (self.limbs[i] == 0);
            }
            result
        }
    }

    trait BigNumTrait {
        fn __is_zero(self) -> bool;
    }

    impl<let N: u32> BigNumTrait for BigNum<N> {
        fn __is_zero(self) -> bool {
            self.__is_zero_impl()
            ^^^^^^^^^^^^^^^^^^^ Call to unconstrained function is unsafe and must be in an unconstrained function or unsafe block
        }
    }
    "#;
    check_errors(src);
}

#[test]
fn cannot_pass_unconstrained_function_to_regular_function() {
    let src = r#"
    fn main() {
        let func = foo;
        expect_regular(func);
                       ^^^^ Converting an unconstrained fn to a non-unconstrained fn is unsafe
    }

    unconstrained fn foo() {}

    fn expect_regular(_func: fn() -> ()) {
    }
    "#;
    check_errors(src);
}

#[test]
fn cannot_assign_unconstrained_and_regular_fn_to_variable() {
    let src = r#"
    fn main() {
        let _func = if true { foo } else { bar };
                                           ^^^ Expected type fn() -> (), found type unconstrained fn() -> ()
    }

    fn foo() {}
    unconstrained fn bar() {}
    "#;
    check_errors(src);
}

#[test]
fn can_pass_regular_function_to_unconstrained_function() {
    let src = r#"
    fn main() {
        let func = foo;
        expect_unconstrained(func);
    }

    fn foo() {}

    fn expect_unconstrained(_func: unconstrained fn() -> ()) {}
    "#;
    assert_no_errors(src);
}

#[test]
fn cannot_pass_unconstrained_function_to_constrained_function() {
    let src = r#"
    fn main() {
        let func = foo;
        expect_regular(func);
                       ^^^^ Converting an unconstrained fn to a non-unconstrained fn is unsafe
    }

    unconstrained fn foo() {}

    fn expect_regular(_func: fn() -> ()) {}
    "#;
    check_errors(src);
}

#[test]
fn cannot_return_function_from_unconstrained_to_constrained() {
    let src = r#"
    fn main() {
        // safety:
        unsafe {
            let _func = make_func();
                        ^^^^^^^^^^^ Functions cannot be returned from an unconstrained runtime to a constrained runtime
        }
    }

    unconstrained fn make_func() -> fn() -> () {
        || {}
    }
    "#;
    check_errors(src);
}

#[test]
fn can_assign_regular_function_to_unconstrained_function_in_explicitly_typed_var() {
    let src = r#"
    fn main() {
        let _func: unconstrained fn() -> () = foo;
    }

    fn foo() {}
    "#;
    assert_no_errors(src);
}

#[test]
fn can_assign_regular_function_to_unconstrained_function_in_struct_member() {
    let src = r#"
    fn main() {
        let _ = Foo { func: foo };
    }

    fn foo() {}

    struct Foo {
        func: unconstrained fn() -> (),
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_unconstrained_methods_typechecked_correctly() {
    // This test checks that we properly track whether a method has been declared as unconstrained on the trait definition
    // and preserves that through typechecking.
    let src = r#"
        trait Foo {
            unconstrained fn identity(self) -> Self {
                self
            }

            unconstrained fn foo(self) -> Field;
        }

        impl Foo for u64 {
            unconstrained fn foo(self) -> Field {
                self as Field
            }
        }

        unconstrained fn main() {
            assert_eq(2.foo(), 2.identity() as Field);
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn warns_on_unneeded_unsafe() {
    let src = r#"
    fn main() {
        // Safety: test
        unsafe {
        ^^^^^^ Unnecessary `unsafe` block
            foo()
        }
    }

    fn foo() {}
    "#;
    check_errors(src);
}

#[test]
fn warns_on_nested_unsafe() {
    let src = r#"
    fn main() {
        // Safety: test
        unsafe {
            // Safety: test
            unsafe {
            ^^^^^^ Unnecessary `unsafe` block
            ~~~~~~ Because it's nested inside another `unsafe` block
                foo()
            }
        }
    }

    unconstrained fn foo() {}
    "#;
    check_errors(src);
}

#[test]
fn no_warns_on_needed_unsafe_with_unneeded_nested() {
    let src = r#"
    fn main() {
        // Safety: test
        unsafe {
            foo();
            // Safety: test
            unsafe {
            ^^^^^^ Unnecessary `unsafe` block
            ~~~~~~ Because it's nested inside another `unsafe` block
                bar();
            }
        }
    }

    unconstrained fn foo() {}

    fn bar() {}
    "#;
    check_errors(src);
}

#[test]
fn deny_inline_attribute_on_unconstrained() {
    let src = r#"
        #[no_predicates]
        ^^^^^^^^^^^^^^^^ misplaced #[no_predicates] attribute on unconstrained function foo. Only allowed on constrained functions
        ~~~~~~~~~~~~~~~~ misplaced #[no_predicates] attribute
        unconstrained pub fn foo(x: Field, y: Field) {
            assert(x != y);
        }
    "#;
    check_errors(src);
}

#[test]
fn deny_fold_attribute_on_unconstrained() {
    let src = r#"
        #[fold]
        ^^^^^^^ misplaced #[fold] attribute on unconstrained function foo. Only allowed on constrained functions
        ~~~~~~~ misplaced #[fold] attribute
        unconstrained pub fn foo(x: Field, y: Field) {
            assert(x != y);
        }
    "#;
    check_errors(src);
}

#[test]
fn deny_inline_never_attribute_on_constrained() {
    let src = r#"
        #[inline_never]
        ^^^^^^^^^^^^^^^ misplaced #[inline_never] attribute on constrained function foo. Only allowed on unconstrained functions
        ~~~~~~~~~~~~~~~ misplaced #[inline_never] attribute
        pub fn foo(x: Field, y: Field) {
            assert(x != y);
        }
    "#;
    check_errors(src);
}

#[test]
fn deny_no_predicates_attribute_on_entry_point() {
    let src = r#"
        #[no_predicates]
        ^^^^^^^^^^^^^^^^ #[no_predicates] attribute is not allowed on entry point function main
        ~~~~~~~~~~~~~~~~ #[no_predicates] attribute not allowed on entry points
        fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn deny_abi_attribute_outside_of_contract() {
    let src = r#"

        #[abi(foo)]
        ^^^^^^^^^^^ #[abi(tag)] attributes can only be used in contracts
        ~~~~~~~~~~~ misplaced #[abi(tag)] attribute
        global foo: Field = 1;
    "#;
    check_errors(src);
}

#[test]
fn break_and_continue_in_constrained_fn() {
    let src = r#"
        fn main() {
            for i in 0 .. 10 {
                if i == 2 {
                    continue;
                    ^^^^^^^^^ continue is only allowed in unconstrained functions
                    ~~~~~~~~~ Constrained code must always have a known number of loop iterations
                }
                if i == 5 {
                    break;
                    ^^^^^^ break is only allowed in unconstrained functions
                    ~~~~~~ Constrained code must always have a known number of loop iterations
                }
            }
        }
    "#;
    check_errors(src);
}

#[test]
fn disallows_test_attribute_on_impl_method() {
    // TODO: improve the error location
    let src = "
        pub struct Foo { }

        impl Foo {

#[test]
            fn foo() { }
               ^^^ The `#[test]` attribute is disallowed on `impl` methods
        }
    ";
    check_errors(src);
}

#[test]
fn disallows_test_attribute_on_trait_impl_method() {
    let src = "
        pub trait Trait {
            fn foo() { }
        }

        pub struct Foo { }

        impl Trait for Foo {
            #[test]
            fn foo() { }
               ^^^ The `#[test]` attribute is disallowed on `impl` methods
        }
    ";
    check_errors(src);
}

#[test]
fn disallows_export_attribute_on_impl_method() {
    // TODO: improve the error location
    let src = "
        pub struct Foo { }

        impl Foo {
            #[export]
            fn foo() { }
               ^^^ The `#[export]` attribute is disallowed on `impl` methods
        }
    ";
    check_errors(src);
}

#[test]
fn disallows_export_attribute_on_trait_impl_method() {
    // TODO: improve the error location
    let src = "
        pub trait Trait {
            fn foo() { }
        }

        pub struct Foo { }

        impl Trait for Foo {
            #[export]
            fn foo() { }
               ^^^ The `#[export]` attribute is disallowed on `impl` methods
        }
    ";
    check_errors(src);
}

#[test]
fn regression_10413() {
    let src = "
    fn main() {
        foo(());
    }

    #[fold]
    fn foo(_: ()) {}
              ^^ Invalid type found in the entry point to a program
              ~~ Unit is not a valid entry point type
    ";
    check_errors(src);
}
