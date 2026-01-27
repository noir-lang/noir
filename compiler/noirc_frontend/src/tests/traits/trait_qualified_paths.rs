//! Tests for qualified path syntax (`<T as Trait>`) and `Self` type usage.
//! Validates disambiguation of trait methods, associated item access, and trait renaming during imports.

use crate::test_utils::GetProgramOptions;
use crate::test_utils::get_program_with_options;
use crate::tests::{
    assert_no_errors, assert_no_errors_without_report, check_errors, check_monomorphization_error,
};

#[test]
fn as_trait_path_in_expression() {
    let src = r#"
        fn main() {
            cursed::<S>();
        }

        fn cursed<T>()
            where T: Foo + Foo2
        {
            <T as Foo>::bar(1);
            <T as Foo2>::bar(());

            // Use each function with different generic arguments
            <T as Foo>::bar(());
        }

        trait Foo  { fn bar<U>(x: U); }
        trait Foo2 { fn bar<U>(x: U); }

        pub struct S {}

        impl Foo for S {
            fn bar<Z>(_x: Z) {}
        }

        impl Foo2 for S {
            fn bar<Z>(_x: Z) {}
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn as_trait_path_called_multiple_times_for_different_t_1() {
    let src = r#"
    pub trait Trait {
        let N: u32;
    }
    impl Trait for Field {
        let N: u32 = 1;
    }
    impl Trait for i32 {
        let N: u32 = 999;
    }
    pub fn load<T>()
    where
        T: Trait,
    {
        let _ = <T as Trait>::N;
    }
    fn main() {
        let _ = load::<Field>();
        let _ = load::<i32>();
    }
    "#;
    check_monomorphization_error(src);
}

#[test]
fn as_trait_path_called_multiple_times_for_different_t_2() {
    let src = r#"
    pub trait Trait {
        let N: u32;
    }
    impl Trait for Field {
        let N: u32 = 1;
    }
    impl Trait for i32 {
        let N: u32 = 999;
    }
    pub fn load<T>()
    where
        T: Trait,
    {
        let _ = T::N;
    }
    fn main() {
        let _ = load::<Field>();
        let _ = load::<i32>();
    }
    "#;
    check_monomorphization_error(src);
}

#[test]
fn as_trait_path_syntax_resolves_outside_impl() {
    let src = r#"
    trait Foo {
        type Assoc;
    }

    struct Bar {}

    impl Foo for Bar {
        type Assoc = i32;
    }

    fn main() {
        // AsTraitPath syntax is a bit silly when associated types
        // are explicitly specified
        let _: i64 = 1 as <Bar as Foo<Assoc = i32>>::Assoc;
                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Expected type i64, found type i32

        let _ = Bar {}; // silence Bar never constructed warning
    }
    "#;
    check_errors(src);
}

#[test]
fn as_trait_path_syntax_no_impl() {
    let src = r#"
    trait Foo {
        type Assoc;
    }

    struct Bar {}

    impl Foo for Bar {
        type Assoc = i32;
    }

    fn main() {
        let _: i64 = 1 as <Bar as Foo<Assoc = i8>>::Assoc;
                                  ^^^ No matching impl found for `Bar: Foo<Assoc = i8>`
                                  ~~~ No impl for `Bar: Foo<Assoc = i8>`

        let _ = Bar {}; // silence Bar never constructed warning
    }
    "#;
    check_errors(src);
}

#[test]
fn does_not_crash_on_as_trait_path_with_empty_path() {
    let src = r#"
        struct Foo {
            x: <N>,
        }
    "#;

    let options = GetProgramOptions { allow_parser_errors: true, ..Default::default() };
    let (_, _, errors) = get_program_with_options(src, options);
    assert!(!errors.is_empty());
}

#[test]
fn uses_self_type_inside_trait() {
    let src = r#"
    trait Foo {
        fn foo() -> Self {
            Self::bar()
        }

        fn bar() -> Self;
    }

    impl Foo for Field {
        fn bar() -> Self {
            1
        }
    }

    fn main() {
        let _: Field = Foo::foo();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn uses_self_type_in_trait_where_clause() {
    let src = r#"
    pub trait Trait {
        fn trait_func(self) -> bool;
    }

    pub trait Foo where Self: Trait {
                              ~~~~~ required by this bound in `Foo`
        fn foo(self) -> bool {
            self.trait_func()
            ^^^^^^^^^^^^^^^^^ No method named 'trait_func' found for type 'Bar'
        }
    }

    struct Bar {}

    impl Foo for Bar {
                 ^^^ The trait bound `_: Trait` is not satisfied
                 ~~~ The trait `Trait` is not implemented for `_`

    }

    fn main() {
        let _ = Bar {}; // silence Bar never constructed warning
    }
    "#;
    check_errors(src);
}

#[test]
fn allows_renaming_trait_during_import() {
    // Regression test for https://github.com/noir-lang/noir/issues/7632
    let src = r#"
    mod trait_mod {
        pub trait Foo {
            fn foo(_: Self) {}
        }

        impl Foo for Field {}
    }

    use trait_mod::Foo as FooTrait;

    fn main(x: Field) {
        x.foo();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn renaming_trait_avoids_name_collisions() {
    // Regression test for https://github.com/noir-lang/noir/issues/7632
    let src = r#"
    mod trait_mod {
        pub trait Foo {
            fn foo(_: Self) {}
        }

        impl Foo for Field {}
    }

    use trait_mod::Foo as FooTrait;

    pub struct Foo {}

    fn main(x: Field) {
        x.foo();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn as_trait_path_self_type() {
    let src = r#"
    pub trait BigCurve<B> {
        fn one() -> Self;
    }

    struct Bn254 {}

    impl<B> BigCurve<B> for Bn254 {
        fn one() -> Self { Bn254 {} }
    }

    fn main() {
        let _ = <Bn254 as BigCurve<()>>::one();
    }
    "#;
    assert_no_errors(src);
}

/// TODO(https://github.com/noir-lang/noir/issues/9562): Reactivate once the issue is resolved
#[test]
#[should_panic(expected = "Expected no errors")]
fn as_trait_path_with_method_turbofish() {
    let src = r#"
    trait Foo {
        fn bar<U>(x: U) -> U;
    }

    impl Foo for u32 {
        fn bar<U>(x: U) -> U { x }
    }

    fn main() {
        let _x: i32 = <u32 as Foo>::bar(42);
        // Explicitly specify U instead of relying on inference
        let _x: i32 = <u32 as Foo>::bar::<i32>(42);
    }
    "#;
    // TODO(https://github.com/noir-lang/noir/issues/9562): use `assert_no_errors` once the issue is resolved
    // assert_no_errors(src);
    assert_no_errors_without_report(src);
}

/// TODO(https://github.com/noir-lang/noir/issues/10436): Reactivate once the issue is resolved
#[test]
#[should_panic(expected = "Expected no errors")]
fn self_with_associated_type_method_call_on_non_primitives() {
    // In Rust, this would be valid:
    // trait MyTrait {
    //     type AssocType;
    // }
    // impl MyTrait for u32 {
    //     type AssocType = Vec<i32>;
    //     fn method() {
    //         Self::AssocType::new()  // Valid in Rust
    //     }
    // }
    let src = r#"
    trait Default {
        fn default() -> Self;
    }

    impl Default for Field {
        fn default() -> Field { 0 }
    }

    trait MyTrait {
        type AssocType;
        fn method() -> Field;
    }

    struct MyStruct { }

    impl MyTrait for MyStruct {
        type AssocType = Field;

        fn method() -> Field {
            // This would work in Rust but not in Noir
            Self::AssocType::default()
        }
    }

    fn main() {
        let _ = MyStruct { };
    }
    "#;
    // TODO(https://github.com/noir-lang/noir/issues/10436): use `assert_no_errors` once the issue is resolved
    // assert_no_errors(src);
    assert_no_errors_without_report(src);
}

/// TODO(https://github.com/noir-lang/noir/issues/10434): Reactivate once the issue is resolved
#[test]
#[should_panic(expected = "Expected no errors")]
fn self_with_associated_type_method_call_on_primitive() {
    // In Noir, the special Self:: handling for primitives only works with
    // exactly 2 segments (Self::method or Self::AssociatedConstant).
    // Paths with 3+ segments fall through to regular path resolution which
    // cannot resolve Self as a path component for primitive types.
    let src = r#"
    trait Default {
        fn default() -> Self;
    }

    impl Default for Field {
        fn default() -> Field { 0 }
    }

    trait MyTrait {
        type AssocType;
        fn method() -> Field;
    }

    impl MyTrait for u32 {
        type AssocType = Field;

        fn method() -> Field {
            // This would work in Rust but not in Noir
            Self::AssocType::default()
        }
    }

    fn main() {}
    "#;
    // TODO(https://github.com/noir-lang/noir/issues/10434): use `assert_no_errors` once the issue is resolved
    // assert_no_errors(src);
    assert_no_errors_without_report(src);
}

/// TODO(https://github.com/noir-lang/noir/issues/10435): Improve error message
#[test]
fn self_with_non_associated_item_access() {
    let src = r#"
    struct Outer {
        inner: Inner
    }

    struct Inner {}

    impl Inner {
        fn method() -> u32 { 42 }
    }

    trait MyTrait {
        fn test() -> u32;
    }

    impl MyTrait for Outer {
        fn test() -> u32 {
            Self::inner::method()
                  ^^^^^ Could not resolve 'inner' in path
        }
    }

    fn main() {
        let inner = Inner {};
        let _ = Outer { inner };
    }
    "#;
    check_errors(src);
}

#[test]
fn self_recursive_call_primitive_in_trait_impl() {
    // Self:: works correctly in recursive calls on primitive types
    let src = r#"
    trait Factorial {
        fn factorial(n: u32) -> u32;
    }

    impl Factorial for u32 {
        fn factorial(n: u32) -> u32 {
            if n <= 1 {
                1
            } else {
                n * Self::factorial(n - 1)
            }
        }
    }

    fn main() {
        assert(u32::factorial(5) == 120);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn self_resolves_correctly_when_multiple_trait_impls_exist() {
    // When a type has multiple trait impls with the same method name,
    // Self:: should resolve to the method in the current impl context
    let src = r#"
    trait MyTrait<T> {
        fn foo(self) -> T;
    }

    impl MyTrait<Field> for u32 {
        fn foo(self) -> Field {
            self as Field
        }
    }

    impl MyTrait<i32> for u32 {
        fn foo(self) -> i32 {
            // Self::foo here should refer to this impl's `foo` method
            if self == 0 {
                0
            } else {
                Self::foo(self - 1) + 1
            }
        }
    }

    fn main() {
        let x: u32 = 5;
        let _: Field = MyTrait::<Field>::foo(x);
        let _: i32 = MyTrait::<i32>::foo(x);
    }
    "#;
    assert_no_errors(src);
}
