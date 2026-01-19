//! Tests for default method implementations in trait definitions.
//! Validates type checking and usage of `Self` within default trait methods.

use crate::tests::{assert_no_errors, check_errors};

#[test]
fn test_impl_self_within_default_def() {
    let src = "
    trait Bar {
        fn ok(self) -> Self;

        fn ref_ok(self) -> Self {
            self.ok()
        }
    }

    impl<T> Bar for (T, T) where T: Bar {
        fn ok(self) -> Self {
            self
        }
    }
    ";
    assert_no_errors(src);
}

#[test]
fn type_checks_trait_default_method_and_errors() {
    let src = r#"
        pub trait Foo {
            fn foo(self) -> i32 {
                            ^^^ expected type i32, found type bool
                            ~~~ expected i32 because of return type
                let _ = self;
                true
                ~~~~ bool returned here
            }
        }
    "#;
    check_errors(src);
}

#[test]
fn type_checks_trait_default_method_and_does_not_error() {
    let src = r#"
        pub trait Foo {
            fn foo(self) -> i32 {
                let _ = self;
                1
            }
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn type_checks_trait_default_method_and_does_not_error_using_self() {
    let src = r#"
        pub trait Foo {
            fn foo(self) -> i32 {
                self.bar()
            }

            fn bar(self) -> i32 {
                let _ = self;
                1
            }
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_with_same_generic_in_different_default_methods() {
    let src = r#"
    pub trait Trait {
        fn foo<let U: u32>(self, _msg: str<U>) {
            let _ = self;
        }

        fn bar<let U: u32>(self, _msg: str<U>) {
            let _ = self;
        }
    }

    pub struct Struct {}

    impl Trait for Struct {}

    pub fn main() {
        Struct {}.bar("Hello");
    }
    "#;
    assert_no_errors(src);
}
