//! Tests for qualified path syntax (`<T as Trait>`) and `Self` type usage.
//! Validates disambiguation of trait methods, associated item access, and trait renaming during imports.

use crate::tests::{assert_no_errors, check_errors, check_monomorphization_error};
use crate::{elaborator::FrontendOptions, test_utils::get_program_with_options};

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

    let allow_parser_errors = true;
    let options = FrontendOptions::test_default();
    let (_, _, errors) = get_program_with_options(src, allow_parser_errors, options);
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
