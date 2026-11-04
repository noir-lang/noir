use crate::tests::{assert_no_errors, check_errors};

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
fn deprecated_trait_method_with_deny_errors_on_method_call_on_concrete_type() {
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
        ^^^^^^^^^^ use something else
        ~~~~~~~~~~ `foo` has been deprecated
    }
    "#;
    check_errors(src);
}

#[test]
fn deprecated_impl_method_warns_with_its_own_note_over_the_trait_methods_note() {
    let src = r#"
    trait Foo {
        #[deprecated("note on the trait method")]
        fn foo(self);
    }

    struct Bar {}

    impl Foo for Bar {
        #[deprecated("note on the impl method")]
        fn foo(self) {}
    }

    fn main() {
        Bar {}.foo();
        ^^^^^^^^^^ note on the impl method
        ~~~~~~~~~~ `foo` has been deprecated
    }
    "#;
    check_errors(src);
}

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
