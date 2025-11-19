//! Tests for trait alias desugaring and validation.
//! Trait aliases allow defining shortcuts for trait bounds like `trait Baz = Foo + Bar`.

use crate::tests::{assert_no_errors, check_errors};

#[test]
fn trait_alias_single_member() {
    let src = r#"
        trait Foo {
            fn foo(self) -> Self;
        }

        trait Baz = Foo;

        impl Foo for Field {
            fn foo(self) -> Self { self }
        }

        fn baz<T>(x: T) -> T where T: Baz {
            x.foo()
        }

        fn main() {
            let x: Field = 0;
            let _ = baz(x);
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_alias_two_members() {
    let src = r#"
        pub trait Foo {
            fn foo(self) -> Self;
        }

        pub trait Bar {
            fn bar(self) -> Self;
        }

        pub trait Baz = Foo + Bar;

        fn baz<T>(x: T) -> T where T: Baz {
            x.foo().bar()
        }

        impl Foo for Field {
            fn foo(self) -> Self {
                self + 1
            }
        }

        impl Bar for Field {
            fn bar(self) -> Self {
                self + 2
            }
        }

        fn main() {
            assert(0.foo().bar() == baz(0));
        }"#;

    assert_no_errors(src);
}

#[test]
fn trait_alias_polymorphic_inheritance() {
    let src = r#"
        trait Foo {
            fn foo(self) -> Self;
        }

        trait Bar<T> {
            fn bar(self) -> T;
        }

        trait Baz<T> = Foo + Bar<T>;

        fn baz<T, U>(x: T) -> U where T: Baz<U> {
            x.foo().bar()
        }

        impl Foo for Field {
            fn foo(self) -> Self {
                self + 1
            }
        }

        impl Bar<bool> for Field {
            fn bar(self) -> bool {
                true
            }
        }

        fn main() {
            assert(0.foo().bar() == baz(0));
        }"#;

    assert_no_errors(src);
}

// TODO(https://github.com/noir-lang/noir/issues/6467): currently failing, so
// this just tests that the trait alias has an equivalent error to the expected
// desugared version
#[test]
fn trait_alias_with_where_clause_has_equivalent_errors() {
    let src = r#"
        trait Bar {
            fn bar(self) -> Self;
        }

        trait Baz {
            fn baz(self) -> bool;
        }

        trait Qux<T>: Bar where T: Baz {}

        impl<T, U> Qux<T> for U where
            U: Bar,
            T: Baz,
        {}

        pub fn qux<T, U>(x: T, _: U) -> bool where U: Qux<T> {
            x.baz()
            ^^^^^^^ No method named 'baz' found for type 'T'
        }
    "#;
    check_errors(src);
}

// TODO(https://github.com/noir-lang/noir/issues/6467): currently failing, so
// this just tests that the trait alias has an equivalent error to the expected
// desugared version
#[test]
fn trait_alias_with_where_clause_has_equivalent_errors_2() {
    let alias_src = r#"
        trait Bar {
            fn bar(self) -> Self;
        }

        trait Baz {
            fn baz(self) -> bool;
        }

        trait Qux<T> = Bar where T: Baz;

        pub fn qux<T, U>(x: T, _: U) -> bool where U: Qux<T> {
            x.baz()
            ^^^^^^^ No method named 'baz' found for type 'T'
        }
    "#;
    check_errors(alias_src);
}
