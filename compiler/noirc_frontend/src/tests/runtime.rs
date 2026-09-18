//! Noir supports multiple runtime environments. This module contains tests related to runtime boundaries and entry point creation.
//! "Runtime boundaries" can refer to calls across the unconstrained/constrained boundary, valid attributes in vanilla programs vs. contracts, defining program entry points, etc.

use crate::tests::{assert_no_errors, check_errors, check_monomorphization_error};

#[test]
fn cannot_call_unconstrained_function_outside_of_unsafe() {
    let src = r#"
    fn main() {
        foo();
        ^^^^^ Call to unconstrained function from constrained function is unsafe and must be in an unconstrained function or unsafe block
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
        ^^^^^^ Call to unconstrained function from constrained function is unsafe and must be in an unconstrained function or unsafe block
        inner(func);
    }

    fn inner(x: unconstrained fn() -> ()) {
        x();
        ^^^ Call to unconstrained function from constrained function is unsafe and must be in an unconstrained function or unsafe block
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
            ^^^^^^^^^^^^^^^^^^^ Call to unconstrained function from constrained function is unsafe and must be in an unconstrained function or unsafe block
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
fn deny_inline_attribute_on_unconstrained_trait_method() {
    let src = r#"
        pub trait Foo {
            #[no_predicates]
            ^^^^^^^^^^^^^^^^ misplaced #[no_predicates] attribute on unconstrained function foo. Only allowed on constrained functions
            ~~~~~~~~~~~~~~~~ misplaced #[no_predicates] attribute
            unconstrained fn foo(x: Field, y: Field) {
                assert(x != y);
            }
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
fn deny_abi_attribute_on_global_outside_contract() {
    let src = r#"
        #[abi(foo)]
        ^^^^^^^^^^^ #[abi(tag)] attributes can only be used in contracts
        ~~~~~~~~~~~ misplaced #[abi(tag)] attribute
        global foo: Field = 1;
    "#;
    check_errors(src);
}

#[test]
fn allow_abi_attribute_on_global_inside_contract() {
    let src = r#"
    contract moo {
        #[abi(foo)]
        global foo: Field = 1;
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn deny_abi_attribute_on_global_with_non_abi_type() {
    let src = r#"
    contract moo {
        #[abi(foo)]
        global foo: () = ();
                    ^^ Globals marked with `#[abi(tag)]` must have an ABI-compatible type
                    ~~ Unit is not a valid ABI type
    }
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
    let src = "
        pub struct Foo { }

        impl Foo {
            #[test]
            ^^^^^^^ The `#[test]` attribute is disallowed on associated functions
            fn foo() { }
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
            ^^^^^^^ The `#[test]` attribute is disallowed on associated functions
            fn foo() { }
        }
    ";
    check_errors(src);
}

#[test]
fn disallows_fuzz_attribute_on_impl_method() {
    let src = "
        pub struct Foo { }

        impl Foo {
            #[fuzz]
            ^^^^^^^ The `#[fuzz]` attribute is disallowed on associated functions
            fn foo(x: u32) { let _ = x; }
        }
    ";
    check_errors(src);
}

#[test]
fn disallows_fuzz_attribute_on_trait_impl_method() {
    let src = "
        pub trait Trait {
            fn foo(x: u32);
        }

        pub struct Foo { }

        impl Trait for Foo {
            #[fuzz]
            ^^^^^^^ The `#[fuzz]` attribute is disallowed on associated functions
            fn foo(x: u32) { let _ = x; }
        }
    ";
    check_errors(src);
}

#[test]
fn disallows_test_attribute_on_trait_definition_method() {
    let src = "
        pub trait Trait {
            #[test]
            ^^^^^^^ The `#[test]` attribute is disallowed on associated functions
            fn foo();
        }
    ";
    check_errors(src);
}

#[test]
fn disallows_test_attribute_on_trait_definition_default_method() {
    let src = "
        pub trait Trait {
            #[test]
            ^^^^^^^ The `#[test]` attribute is disallowed on associated functions
            fn foo() { }
        }
    ";
    check_errors(src);
}

#[test]
fn disallows_fuzz_attribute_on_trait_definition_method() {
    let src = "
        pub trait Trait {
            #[fuzz]
            ^^^^^^^ The `#[fuzz]` attribute is disallowed on associated functions
            fn foo(x: u32);
        }
    ";
    check_errors(src);
}

#[test]
fn disallows_fuzz_attribute_on_trait_definition_default_method() {
    let src = "
        pub trait Trait {
            #[fuzz]
            ^^^^^^^ The `#[fuzz]` attribute is disallowed on associated functions
            fn foo(x: u32) { let _ = x; }
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
            pub fn foo() { }
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

#[test]
fn user_defined_verify_proof_with_type_is_allowed_in_brillig() {
    // User-defined functions having verify_proof_with_type name should not error.
    // The lint only applies to std::verify_proof_with_type from the standard library.
    let src = r#"
    unconstrained fn main() {
        let verification_key: [Field; 114] = [0; 114];
        let proof: [Field; 94] = [0; 94];
        let public_inputs: [Field; 1] = [0];
        let key_hash: Field = 0;
        let proof_type: u32 = 0;

        // This is OK: it's a user-defined function, not std::verify_proof_with_type
        verify_proof_with_type(verification_key, proof, public_inputs, key_hash, proof_type);
    }

    fn verify_proof_with_type<let N: u32, let M: u32, let K: u32>(
        _verification_key: [Field; N],
        _proof: [Field; M],
        _public_inputs: [Field; K],
        _key_hash: Field,
        _proof_type: u32,
    ) {}
    "#;
    assert_no_errors(src);
}

#[test]
fn cannot_return_vector_from_unconstrained_to_constrained() {
    let src = r#"
    unconstrained fn clear() -> [u32] {
        @[1, 2, 3]
    }

    fn main() {
        // Safety: testing
        let _x = unsafe { clear() };
                          ^^^^^^^ Vectors cannot be returned from an unconstrained runtime to a constrained runtime
    }
    "#;
    check_errors(src);
}

#[test]
fn regression_10259() {
    let src = r#"
    unconstrained fn foo<T>(x: T) -> T {
        x
    }

    fn bar<T>(x: T) -> T {
        // Safety: testing
        unsafe {
            foo(x)
            ^^^^^^ Vector `[Field]` cannot be returned from an unconstrained runtime to a constrained runtime
        }
    }

    fn main(x: Field) {
        let xs = @[x, x + 1, x + 2];
        let _ = bar(xs);
    }
    "#;
    check_monomorphization_error(src);
}

/// Globals are evaluated in a `comptime` context so they can call unconstrained functions without `unsafe` blocks
#[test]
fn call_unconstrained_function_in_lambda_in_global() {
    let src = r#"
    pub global foo: fn() = || bar();
    unconstrained fn bar() {}

    fn main() {}
    "#;
    assert_no_errors(src);
}

#[test]
fn cannot_assign_unconstrained_fn_to_constrained_slot_inside_unconstrained_call_args() {
    let src = r#"
    fn main() {
        let mut g: fn() -> () = foo;
        // Safety: testing
        unsafe { sink({ g = bar; 0 }) };
                            ^^^ Converting an unconstrained fn to a non-unconstrained fn is unsafe
        g();
    }

    fn foo() {}
    unconstrained fn bar() {}
    unconstrained fn sink<T>(_v: T) {}
    "#;
    check_errors(src);
}

#[test]
fn cannot_assign_unconstrained_fn_to_constrained_struct_field_inside_unconstrained_call_args() {
    let src = r#"
    fn main() {
        let mut c = Cfg { f: foo };
        // Safety: testing
        unsafe { sink({ c.f = bar; 0 }) };
                              ^^^ Converting an unconstrained fn to a non-unconstrained fn is unsafe
        (c.f)();
    }

    struct Cfg {
        f: fn() -> (),
    }

    fn foo() {}
    unconstrained fn bar() {}
    unconstrained fn sink<T>(_v: T) {}
    "#;
    check_errors(src);
}

#[test]
fn cannot_let_bind_unconstrained_fn_to_constrained_slot_inside_unconstrained_call_args() {
    let src = r#"
    fn main() {
        // Safety: testing
        unsafe { sink({ let _g: fn() -> () = bar; 0 }) };
                                             ^^^ Converting an unconstrained fn to a non-unconstrained fn is unsafe
    }

    unconstrained fn bar() {}
    unconstrained fn sink<T>(_v: T) {}
    "#;
    check_errors(src);
}

#[test]
fn cannot_build_struct_with_unconstrained_fn_field_inside_unconstrained_call_args() {
    let src = r#"
    fn main() {
        // Safety: testing
        unsafe { sink(Cfg { f: bar }) };
                               ^^^ Converting an unconstrained fn to a non-unconstrained fn is unsafe
    }

    struct Cfg {
        f: fn() -> (),
    }

    unconstrained fn bar() {}
    unconstrained fn sink<T>(_v: T) {}
    "#;
    check_errors(src);
}

#[test]
fn cannot_return_unconstrained_fn_as_constrained_fn_inside_unconstrained_call_args() {
    let src = r#"
    fn main() {
        // Safety: testing
        unsafe { sink(make()) };
    }

    fn make() -> fn() -> () {
        bar
        ^^^ Converting an unconstrained fn to a non-unconstrained fn is unsafe
    }

    unconstrained fn bar() {}
    unconstrained fn sink<T>(_v: T) {}
    "#;
    check_errors(src);
}

#[test]
fn can_pass_unconstrained_fn_to_unconstrained_function_expecting_constrained_fn() {
    let src = r#"
    fn main() {
        // Safety: testing
        unsafe { expect_regular(foo) };
    }

    unconstrained fn foo() {}

    unconstrained fn expect_regular(_func: fn() -> ()) {}
    "#;
    assert_no_errors(src);
}

/// A lambda in the argument list of an unconstrained call is elaborated as unconstrained even
/// when it is nested inside a further method call, because a method call does not reset
/// `in_unconstrained_args` the way a plain call does. The mismatch against that method's
/// constrained parameter is one the compiler creates for itself, so it stays exempt.
///
/// `noir-bignum` spells this as `batch_invert_slice(&params, x.map(|bn| bn.get_limbs()))`, where
/// `batch_invert_slice` is unconstrained and `map` is not.
#[test]
fn can_pass_lambda_to_a_method_call_nested_in_unconstrained_call_args() {
    let src = r#"
    fn main(x: Field) {
        let w = Wrapper { value: x };
        // Safety: testing
        unsafe { expect_field(w.apply(|v: Field| v + 1)) };
    }

    struct Wrapper {
        value: Field,
    }

    impl Wrapper {
        fn apply<Env>(self, f: fn[Env](Field) -> Field) -> Field {
            f(self.value)
        }
    }

    unconstrained fn expect_field(_v: Field) {}
    "#;
    assert_no_errors(src);
}

/// The same lambda reached through a plain constrained call instead of a method call: the inner
/// call resets `in_unconstrained_args`, so the lambda is elaborated constrained and matches the
/// parameter with no coercion at all. Nothing is exempted here, and nothing needs to be.
#[test]
fn can_pass_lambda_to_a_constrained_call_nested_in_unconstrained_call_args() {
    let src = r#"
    fn main(x: Field) {
        // Safety: testing
        unsafe { expect_field(apply(x, |v: Field| v + 1)) };
    }

    fn apply<Env>(value: Field, f: fn[Env](Field) -> Field) -> Field {
        f(value)
    }

    unconstrained fn expect_field(_v: Field) {}
    "#;
    assert_no_errors(src);
}

#[test]
fn can_pass_lambda_calling_unconstrained_code_to_unconstrained_function_expecting_constrained_fn() {
    let src = r#"
    fn main() {
        // Safety: testing
        unsafe { expect_regular(|| foo()) };
    }

    unconstrained fn foo() {}

    unconstrained fn expect_regular(_func: fn() -> ()) {}
    "#;
    assert_no_errors(src);
}

/// A struct in a user crate has its fields resolved lazily, so a call elaborated from inside a
/// comptime attribute can reach one whose fields are still deferred. The boundary check resolves
/// what it needs on the way in, in every position a value of the returned type can hide a vector.
#[test]
fn vector_in_deferred_struct_returned_from_unconstrained() {
    let src = r#"
    pub struct Wrapper {
        vector: [Field],
    }

    unconstrained fn consume(w: Wrapper) -> Wrapper {
        w
    }

    #[generate]
    ~~~~~~~~~~~ While running this function attribute
    pub fn trigger() {}

    comptime fn generate(_f: FunctionDefinition) -> Quoted {
        quote {
            pub fn use_it(w: Wrapper) {
                // Safety: testing
                let _ = unsafe { consume(w) };
                                 ^^^^^^^^^^ Vectors cannot be returned from an unconstrained runtime to a constrained runtime
            }
        }
    }

    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn vector_in_deferred_struct_returned_through_a_bound_type_variable() {
    let src = r#"
    pub struct Wrapper {
        vector: [Field],
    }

    fn id<T>(x: T) -> T {
        x
    }

    unconstrained fn consume<U>(u: U) -> U {
        u
    }

    #[generate]
    ~~~~~~~~~~~ While running this function attribute
    pub fn trigger() {}

    comptime fn generate(_f: FunctionDefinition) -> Quoted {
        quote {
            pub fn use_it(w: Wrapper) {
                // Safety: testing
                let _ = unsafe { consume(id(w)) };
                                 ^^^^^^^^^^^^^^ Vectors cannot be returned from an unconstrained runtime to a constrained runtime
            }
        }
    }

    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn vector_in_deferred_struct_returned_in_a_format_string() {
    let src = r#"
    pub struct Wrapper {
        vector: [Field],
    }

    unconstrained fn consume() -> fmtstr<3, (Wrapper,)> {
        let w = Wrapper { vector: @[0] };
        f"{w}"
    }

    #[generate]
    ~~~~~~~~~~~ While running this function attribute
    pub fn trigger() {}

    comptime fn generate(_f: FunctionDefinition) -> Quoted {
        quote {
            pub fn use_it() {
                // Safety: testing
                let _ = unsafe { consume() };
                                 ^^^^^^^^^ Vectors cannot be returned from an unconstrained runtime to a constrained runtime
            }
        }
    }

    fn main() {}
    "#;
    check_errors(src);
}

/// A method call manages `in_unconstrained_args` exactly as a plain call does, so passing a named
/// unconstrained function to an unconstrained callee expecting a constrained `fn(..)` works
/// whichever way the call is spelled.
#[test]
fn can_pass_unconstrained_fn_to_unconstrained_method_expecting_constrained_fn() {
    let src = r#"
    fn main() {
        let s = S {};
        // Safety: testing
        unsafe { s.expect_regular(foo) };
    }

    struct S {}

    impl S {
        unconstrained fn expect_regular(self, _func: fn() -> ()) {
            let _ = self;
        }
    }

    unconstrained fn foo() {}
    "#;
    assert_no_errors(src);
}

/// The lambda counterpart: an unconstrained method infers its lambda argument as unconstrained, so
/// a `&mut` reaching that lambda never crosses a runtime boundary. This is the shape
/// `test_programs/compile_success_no_bug/regression_10631` covers for plain calls.
#[test]
fn can_pass_lambda_taking_mutable_reference_to_unconstrained_method() {
    let src = r#"
    fn main() {
        let s = S {};
        // Safety: testing
        unsafe { s.expect_regular(|v| foo(v)) };
    }

    struct S {}

    impl S {
        unconstrained fn expect_regular(self, _func: fn(&mut u32)) {
            let _ = self;
        }
    }

    unconstrained fn foo(_x: &mut u32) {}
    "#;
    assert_no_errors(src);
}

/// Conversely a *constrained* method resets the flag, so a lambda in its argument list is
/// elaborated constrained even when the method call is itself nested inside an unconstrained
/// call's arguments. Without that reset the lambda is compiled to Brillig and the constrained
/// method dispatches into it, which `check_for_missing_brillig_constraints` reports as a `bug:`.
#[test]
fn lambda_passed_to_constrained_method_inside_unconstrained_call_args_stays_constrained() {
    let src = r#"
    fn main(x: Field) {
        let w = Wrapper { value: x };
        // Safety: testing
        unsafe { expect_field(w.apply(|v: Field| v + 1)) };
    }

    struct Wrapper {
        value: Field,
    }

    impl Wrapper {
        fn apply<Env>(self, f: fn[Env](Field) -> Field) -> Field {
            f(self.value)
        }
    }

    unconstrained fn expect_field(_v: Field) {}
    "#;
    assert_no_errors(src);
}
