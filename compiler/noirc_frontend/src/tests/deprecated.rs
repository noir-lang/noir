use noirc_errors::CustomDiagnostic;

use crate::tests::{assert_no_errors, check_errors, get_program_errors};

fn deprecation_diagnostic(src: &str, message: &str) -> CustomDiagnostic {
    let errors = get_program_errors(src);
    let mut diagnostics = errors.iter().map(CustomDiagnostic::from);
    diagnostics
        .find(|diagnostic| diagnostic.message == message)
        .unwrap_or_else(|| panic!("Expected a diagnostic with message {message:?}"))
}

// Calls on a concrete type resolve to the impl's method, which inherits the deprecation of the trait
// method it implements.

#[test]
fn deprecated_trait_method_warns_on_method_call_on_concrete_type() {
    let src = r#"
    trait Foo {
        #[deprecated("use something else")]
        fn foo(self);
    }

    struct Bar {}

    impl Foo for Bar {
        fn foo(self) {}
    }

    fn main() {
        Bar {}.foo();
        ^^^^^^^^^^ use something else
        ~~~~~~~~~~ `foo` has been deprecated
    }
    "#;
    check_errors(src);
}

#[test]
fn deprecated_trait_method_warns_on_path_call_on_concrete_type() {
    let src = r#"
    trait Foo {
        #[deprecated("use something else")]
        fn foo(self);
    }

    struct Bar {}

    impl Foo for Bar {
        fn foo(self) {}
    }

    fn main() {
        Bar::foo(Bar {});
        ^^^^^^^^ use something else
        ~~~~~~~~ `foo` has been deprecated
    }
    "#;
    check_errors(src);
}

#[test]
fn deprecated_trait_default_method_overridden_in_impl_warns_on_method_call() {
    let src = r#"
    trait Foo {
        #[deprecated("use something else")]
        fn foo(_self: Self) {}
    }

    struct Bar {}

    impl Foo for Bar {
        fn foo(self) {}
    }

    fn main() {
        Bar {}.foo();
        ^^^^^^^^^^ use something else
        ~~~~~~~~~~ `foo` has been deprecated
    }
    "#;
    check_errors(src);
}

#[test]
fn deprecated_trait_method_warns_on_call_in_generic_trait_impl() {
    let src = r#"
    trait Foo<T> {
        #[deprecated("use something else")]
        fn foo(self, x: T);
    }

    struct Bar {}

    impl Foo<u8> for Bar {
        fn foo(self, _x: u8) {}
    }

    fn main() {
        Bar {}.foo(1_u8);
        ^^^^^^^^^^ use something else
        ~~~~~~~~~~ `foo` has been deprecated
    }
    "#;
    check_errors(src);
}

#[test]
fn deprecated_trait_method_warns_on_call_in_impl_for_generic_type() {
    let src = r#"
    trait Foo {
        #[deprecated("use something else")]
        fn foo(self);
    }

    struct Bar<T> { x: T }

    impl<T> Foo for Bar<T> {
        fn foo(self) {
            let _ = self.x;
        }
    }

    fn main() {
        Bar { x: 1_u8 }.foo();
        ^^^^^^^^^^^^^^^^^^^ use something else
        ~~~~~~~~~~~~~~~~~~~ `foo` has been deprecated
    }
    "#;
    check_errors(src);
}

#[test]
fn deprecated_trait_method_warning_is_a_warning() {
    let src = r#"
    trait Foo {
        #[deprecated("use something else")]
        fn foo(self);
    }

    struct Bar {}

    impl Foo for Bar {
        fn foo(self) {}
    }

    fn main() {
        Bar {}.foo();
    }
    "#;
    assert!(deprecation_diagnostic(src, "use something else").is_warning());
}

#[test]
fn deprecated_trait_method_with_deny_is_an_error_on_call_on_concrete_type() {
    let src = r#"
    trait Foo {
        #[deprecated(deny, "use something else")]
        fn foo(self);
    }

    struct Bar {}

    impl Foo for Bar {
        fn foo(self) {}
    }

    fn main() {
        Bar {}.foo();
    }
    "#;
    assert!(deprecation_diagnostic(src, "use something else").is_error());
}

// A method in a trait impl can't be deprecated: the attribute warns, and has no other effect.

#[test]
fn deprecated_on_trait_impl_method_warns_and_calls_stay_silent() {
    let src = r#"
    trait Foo {
        fn foo(self);
    }

    struct Bar {}

    impl Foo for Bar {
        #[deprecated("note on the impl method")]
        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ The `#[deprecated]` attribute has no effect on a method in a trait impl
        ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ Deprecate the method on the trait instead
        fn foo(self) {}
    }

    fn main() {
        Bar {}.foo();
    }
    "#;
    check_errors(src);
}

#[test]
fn deprecated_with_deny_on_trait_impl_method_only_warns() {
    let src = r#"
    trait Foo {
        fn foo(self);
    }

    struct Bar {}

    impl Foo for Bar {
        #[deprecated(deny, "note on the impl method")]
        fn foo(self) {}
    }

    fn main() {
        Bar {}.foo();
    }
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1, "Expected a single diagnostic: {errors:?}");
    assert!(CustomDiagnostic::from(&errors[0]).is_warning());
}

#[test]
fn deprecated_on_trait_impl_method_does_not_replace_the_trait_methods_attribute() {
    let src = r#"
    trait Foo {
        #[deprecated(deny, "note on the trait method")]
        fn foo(self);
    }

    struct Bar {}

    impl Foo for Bar {
        #[deprecated("note on the impl method")]
        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ The `#[deprecated]` attribute has no effect on a method in a trait impl
        ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ Deprecate the method on the trait instead
        fn foo(self) {}
    }

    fn main() {
        Bar {}.foo();
        ^^^^^^^^^^ note on the trait method
        ~~~~~~~~~~ `foo` has been deprecated
    }
    "#;
    check_errors(src);

    let src = r#"
    trait Foo {
        #[deprecated(deny, "note on the trait method")]
        fn foo(self);
    }

    struct Bar {}

    impl Foo for Bar {
        #[deprecated("note on the impl method")]
        fn foo(self) {}
    }

    fn main() {
        Bar {}.foo();
    }
    "#;
    assert!(deprecation_diagnostic(src, "note on the trait method").is_error());
}

#[test]
fn deprecated_on_inherent_method_warns_on_calls() {
    let src = r#"
    struct Bar {}

    impl Bar {
        #[deprecated("use something else")]
        fn foo(_self: Self) {}
    }

    fn main() {
        Bar {}.foo();
        ^^^^^^^^^^ use something else
        ~~~~~~~~~~ `foo` has been deprecated
    }
    "#;
    check_errors(src);
}

// These calls resolve to the trait's own function, which carries the attribute directly.

#[test]
fn deprecated_trait_method_warns_on_qualified_path_call() {
    let src = r#"
    trait Foo {
        #[deprecated("use something else")]
        fn foo(self);
    }

    struct Bar {}

    impl Foo for Bar {
        fn foo(self) {}
    }

    fn main() {
        <Bar as Foo>::foo(Bar {});
                      ^^^ use something else
                      ~~~ `foo` has been deprecated
    }
    "#;
    check_errors(src);
}

#[test]
fn deprecated_trait_method_warns_on_trait_path_call() {
    let src = r#"
    trait Foo {
        #[deprecated("use something else")]
        fn foo(self);
    }

    struct Bar {}

    impl Foo for Bar {
        fn foo(self) {}
    }

    fn main() {
        Foo::foo(Bar {});
        ^^^^^^^^ use something else
        ~~~~~~~~ `foo` has been deprecated
    }
    "#;
    check_errors(src);
}

#[test]
fn deprecated_trait_method_warns_on_call_through_trait_bound() {
    let src = r#"
    trait Foo {
        #[deprecated("use something else")]
        fn foo(self);
    }

    struct Bar {}

    impl Foo for Bar {
        fn foo(self) {}
    }

    fn call_foo<T: Foo>(x: T) {
        x.foo();
        ^^^^^ use something else
        ~~~~~ `foo` has been deprecated
    }

    fn main() {
        call_foo(Bar {});
    }
    "#;
    check_errors(src);
}

#[test]
fn deprecated_trait_default_method_warns_on_method_call_on_concrete_type() {
    let src = r#"
    trait Foo {
        #[deprecated("use something else")]
        fn foo(_self: Self) {}
    }

    struct Bar {}

    impl Foo for Bar {}

    fn main() {
        Bar {}.foo();
        ^^^^^^^^^^ use something else
        ~~~~~~~~~~ `foo` has been deprecated
    }
    "#;
    check_errors(src);
}

// Code that doesn't call a deprecated function stays silent.

#[test]
fn implementing_a_deprecated_trait_method_does_not_warn() {
    let src = r#"
    trait Foo {
        #[deprecated("use something else")]
        fn foo(self);
    }

    struct Bar {}

    impl Foo for Bar {
        fn foo(self) {}
    }

    fn main() {
        let _ = Bar {};
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn other_methods_of_a_trait_with_a_deprecated_method_do_not_warn() {
    let src = r#"
    trait Foo {
        #[deprecated("use something else")]
        fn foo(self);

        fn other(self);
    }

    struct Bar {}

    impl Foo for Bar {
        fn foo(self) {}
        fn other(self) {}
    }

    fn main() {
        Bar {}.other();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn deprecated_trait_method_does_not_warn_for_unrelated_inherent_method_of_same_name() {
    let src = r#"
    trait Foo {
        #[deprecated("use something else")]
        fn foo(self);
    }

    struct Bar {}

    impl Foo for Bar {
        fn foo(self) {}
    }

    impl Bar {
        fn foo(_self: Self) {}
    }

    fn main() {
        Bar {}.foo();
    }
    "#;
    assert_no_errors(src);
}
