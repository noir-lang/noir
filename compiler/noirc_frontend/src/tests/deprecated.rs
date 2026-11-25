use noirc_errors::CustomDiagnostic;

use crate::elaborator::UnstableFeature;
use crate::tests::{
    assert_no_errors, check_errors, check_errors_using_features, get_program_errors,
};

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

#[test]
fn deprecated_on_method_in_macro_generated_trait_impl_warns_and_calls_stay_silent() {
    let src = r#"
    trait Foo {
        fn foo(self);
    }

    #[generate_impl]
    ~~~~~~~~~~~~~~~~ While running this function attribute
    struct Bar {}

    comptime fn generate_impl(_: TypeDefinition) -> Quoted {
        quote {
            impl Foo for Bar {
                #[deprecated("note on the impl method")]
                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ The `#[deprecated]` attribute has no effect on a method in a trait impl
                ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ Deprecate the method on the trait instead
                fn foo(self) {}
            }
        }
    }

    fn main() {
        Bar {}.foo();
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

#[test]
fn deprecated_trait_warns_on_impl() {
    let src = r#"
    #[deprecated("use something else")]
    trait Foo {}

    struct Bar {}

    impl Foo for Bar {}
         ^^^ use something else
         ~~~ `Foo` has been deprecated

    fn main() {
        let _ = Bar {};
    }
    "#;
    check_errors(src);
}

#[test]
fn deprecated_trait_without_note_warns_with_default_message() {
    let src = r#"
    #[deprecated]
    trait Foo {}

    struct Bar {}

    impl Foo for Bar {}
         ^^^ `Foo` has been deprecated

    fn main() {
        let _ = Bar {};
    }
    "#;
    check_errors(src);
}

#[test]
fn deprecated_trait_warns_on_where_clause() {
    let src = r#"
    #[deprecated("use something else")]
    trait Foo {}

    pub fn generic<T>(_x: T) where T: Foo {}
                                      ^^^ use something else
                                      ~~~ `Foo` has been deprecated

    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn deprecated_trait_warns_on_inline_bound() {
    let src = r#"
    #[deprecated("use something else")]
    trait Foo {}

    pub fn generic<T: Foo>(_x: T) {}
                      ^^^ use something else
                      ~~~ `Foo` has been deprecated

    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn deprecated_trait_warns_on_impl_trait_argument() {
    let src = r#"
    #[deprecated("use something else")]
    trait Foo {}

    pub fn generic(_x: impl Foo) {}
                            ^^^ use something else
                            ~~~ `Foo` has been deprecated

    fn main() {}
    "#;
    check_errors_using_features(src, &[UnstableFeature::TraitAsType]);
}

#[test]
fn deprecated_trait_warns_on_parent_bound() {
    let src = r#"
    #[deprecated("use something else")]
    pub trait Foo {}

    pub trait Baz: Foo {}
                   ^^^ use something else
                   ~~~ `Foo` has been deprecated

    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn deprecated_trait_warns_on_associated_type_bound() {
    let src = r#"
    #[deprecated("use something else")]
    pub trait Foo {}

    pub trait Baz {
        type Assoc: Foo;
                    ^^^ use something else
                    ~~~ `Foo` has been deprecated
    }

    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn deprecated_trait_warns_on_inherent_impl_bound() {
    let src = r#"
    #[deprecated("use something else")]
    trait Foo {}

    pub struct Bar<T> { pub x: T }

    impl<T> Bar<T> where T: Foo {
                            ^^^ use something else
                            ~~~ `Foo` has been deprecated
        pub fn get(self) -> T {
            self.x
        }
    }

    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn deprecated_trait_warns_on_method_call_on_concrete_type() {
    let src = r#"
    #[deprecated("use something else")]
    trait Foo {
        fn foo(self);
    }

    struct Bar {}

    impl Foo for Bar {
         ^^^ use something else
         ~~~ `Foo` has been deprecated
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
fn deprecated_trait_warns_on_trait_path_call() {
    let src = r#"
    #[deprecated("use something else")]
    trait Foo {
        fn foo(self);
    }

    struct Bar {}

    impl Foo for Bar {
         ^^^ use something else
         ~~~ `Foo` has been deprecated
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
fn deprecated_trait_warns_on_qualified_path_call() {
    let src = r#"
    #[deprecated("use something else")]
    trait Foo {
        fn foo(self);
    }

    struct Bar {}

    impl Foo for Bar {
         ^^^ use something else
         ~~~ `Foo` has been deprecated
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
fn deprecated_trait_warns_on_default_method_call() {
    let src = r#"
    #[deprecated("use something else")]
    trait Foo {
        fn foo(_self: Self) {}
    }

    struct Bar {}

    impl Foo for Bar {}
         ^^^ use something else
         ~~~ `Foo` has been deprecated

    fn main() {
        Bar {}.foo();
        ^^^^^^^^^^ use something else
        ~~~~~~~~~~ `foo` has been deprecated
    }
    "#;
    check_errors(src);
}

#[test]
fn deprecated_trait_note_gives_way_to_deprecated_method_note() {
    let src = r#"
    #[deprecated("note on the trait")]
    trait Foo {
        #[deprecated("note on the method")]
        fn foo(self);
    }

    struct Bar {}

    impl Foo for Bar {
         ^^^ note on the trait
         ~~~ `Foo` has been deprecated
        fn foo(self) {}
    }

    fn main() {
        Bar {}.foo();
        ^^^^^^^^^^ note on the method
        ~~~~~~~~~~ `foo` has been deprecated
    }
    "#;
    check_errors(src);
}

#[test]
fn deprecated_trait_does_not_warn_inside_its_own_declaration() {
    let src = r#"
    #[deprecated("use something else")]
    pub trait Foo {
        fn foo(self);

        fn calls_sibling(self) {
            self.foo()
        }

        fn self_bound<T: Foo>(_x: T);
    }

    fn main() {}
    "#;
    assert_no_errors(src);
}

#[test]
fn deprecated_trait_warns_inside_its_impls() {
    let src = r#"
    #[deprecated("use something else")]
    trait Foo {
        fn foo(self);
        fn sibling(_self: Self) {}
    }

    struct Bar {}

    impl Foo for Bar {
         ^^^ use something else
         ~~~ `Foo` has been deprecated
        fn foo(self) {
            self.sibling()
            ^^^^^^^^^^^^ use something else
            ~~~~~~~~~~~~ `sibling` has been deprecated
        }
    }

    fn main() {
        let _ = Bar {};
    }
    "#;
    check_errors(src);
}

#[test]
fn deprecated_trait_warns_from_another_deprecated_item() {
    let src = r#"
    #[deprecated("use something else")]
    pub trait Foo {}

    #[deprecated("also old")]
    pub trait Baz: Foo {}
                   ^^^ use something else
                   ~~~ `Foo` has been deprecated

    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn deprecated_trait_does_not_warn_for_an_inherent_method_of_the_same_name() {
    let src = r#"
    #[deprecated("use something else")]
    pub trait Foo {
        fn foo(self);
    }

    struct Bar {}

    impl Bar {
        fn foo(_self: Self) {}
    }

    fn main() {
        Bar {}.foo();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn deprecated_trait_alias_warns_on_use_but_not_on_its_declaration() {
    let src = r#"
    pub trait Foo {}

    #[deprecated("use something else")]
    pub trait Baz = Foo;

    pub fn generic<T: Baz>(_x: T) {}
                      ^^^ use something else
                      ~~~ `Baz` has been deprecated

    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn deprecated_trait_warns_on_as_trait_path_in_type() {
    let src = r#"
    #[deprecated("use something else")]
    pub trait Foo {
        type Assoc;
    }

    pub struct Bar {}

    pub fn f(_x: <Bar as Foo>::Assoc) {}
                         ^^^ use something else
                         ~~~ `Foo` has been deprecated

    impl Foo for Bar {
         ^^^ use something else
         ~~~ `Foo` has been deprecated
        type Assoc = u32;
    }

    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn deprecated_trait_does_not_warn_on_as_trait_path_inside_its_own_declaration() {
    let src = r#"
    #[deprecated("use something else")]
    pub trait Foo {
        type Assoc;

        fn get(self) -> <Self as Foo>::Assoc;
    }

    fn main() {}
    "#;
    assert_no_errors(src);
}

#[test]
fn deprecated_trait_warns_on_impl_generated_by_a_macro() {
    let src = r#"
    #[deprecated("use something else")]
    pub trait Foo {}

    #[generate_impl]
    ~~~~~~~~~~~~~~~~ While running this function attribute
    pub struct Bar {}

    comptime fn generate_impl(_: TypeDefinition) -> Quoted {
        quote {
            impl Foo for Bar {}
                 ^^^ use something else
                 ~~~ `Foo` has been deprecated
        }
    }

    fn main() {}
    "#;
    check_errors(src);
}

/// `check_errors` matches annotations against a set, so it can't tell one warning from two at the
/// same location. A bound can be resolved more than once, which must still report a single use.
#[test]
fn deprecated_trait_warns_once_per_use() {
    let prelude = r#"
    #[deprecated("old trait")]
    pub trait Foo {
        fn foo(self);
    }
    "#;
    let cases = [
        ("where clause", "pub fn f<T>(_x: T) where T: Foo {}", 1),
        ("inline bound", "pub fn f<T: Foo>(_x: T) {}", 1),
        ("impl trait", "pub fn f(_x: impl Foo) {}", 1),
        ("parent bound", "pub trait Baz: Foo {}", 1),
        ("where on trait", "pub trait Baz where Self: Foo {}", 1),
        ("trait alias", "pub trait Baz = Foo;", 1),
        ("associated type bound", "pub trait Baz { type A: Foo; }", 1),
        ("bound on trait method", "pub trait Baz { fn m<T: Foo>(x: T); }", 1),
        (
            "where clause on an impl with several methods",
            "pub struct S<T> { x: T }
             impl<T> S<T> where T: Foo {
                 pub fn a(self) -> T { self.x }
                 pub fn b(self) -> T { self.x }
             }",
            1,
        ),
        (
            "where clause on a trait impl with several methods",
            "pub trait Baz { fn a(self); fn b(self); }
             pub struct S<T> { x: T }
             impl<T> Baz for S<T> where T: Foo {
                 fn a(self) {}
                 fn b(self) {}
             }",
            1,
        ),
        (
            "bound on a default method inherited by an impl",
            "pub trait Baz { fn m<T: Foo>(_x: T) {} }
             pub struct S {}
             impl Baz for S {}",
            1,
        ),
        (
            "bound on trait impl",
            "pub struct S<T> { x: T } impl<T> Foo for S<T> where T: Foo { fn foo(self) {} }",
            2,
        ),
        (
            "bound in generic struct method",
            "pub struct S {} impl S { pub fn g<T: Foo>(_x: T) {} }",
            1,
        ),
        (
            "qualified method call",
            "pub struct S {} impl Foo for S { fn foo(self) {} } pub fn f() { <S as Foo>::foo(S {}); }",
            2,
        ),
        (
            "where clause on a macro-generated impl with several methods",
            "pub trait Baz { fn a(self); fn b(self); }
             #[generate_impl]
             pub struct S<T> { x: T }
             comptime fn generate_impl(_: TypeDefinition) -> Quoted {
                 quote {
                     impl<T> Baz for S<T> where T: Foo {
                         fn a(self) {}
                         fn b(self) {}
                     }
                 }
             }",
            1,
        ),
        ("two separate uses", "pub fn f<T: Foo>(_x: T) {} pub fn g<T: Foo>(_x: T) {}", 2),
    ];

    for (name, item, expected) in cases {
        let src = format!("{prelude}\n{item}\nfn main() {{}}");
        let warnings = get_program_errors(&src)
            .iter()
            .filter(|error| CustomDiagnostic::from(*error).message == "old trait")
            .count();
        assert_eq!(warnings, expected, "wrong number of warnings for {name}");
    }
}

#[test]
fn deprecated_trait_warning_is_a_warning() {
    let src = r#"
    #[deprecated("use something else")]
    trait Foo {}

    struct Bar {}

    impl Foo for Bar {}

    fn main() {
        let _ = Bar {};
    }
    "#;
    assert!(deprecation_diagnostic(src, "use something else").is_warning());
}

#[test]
fn deprecated_trait_with_deny_is_an_error() {
    let src = r#"
    #[deprecated(deny, "use something else")]
    trait Foo {}

    struct Bar {}

    impl Foo for Bar {}

    fn main() {
        let _ = Bar {};
    }
    "#;
    assert!(deprecation_diagnostic(src, "use something else").is_error());
}

#[test]
fn deprecated_trait_method_does_not_soften_deny_on_the_trait() {
    let src = r#"
    #[deprecated(deny, "note on the trait")]
    trait Foo {
        #[deprecated("note on the method")]
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
    assert!(deprecation_diagnostic(src, "note on the method").is_error());
}

#[test]
fn deprecated_trait_method_without_note_uses_the_traits_note() {
    let src = r#"
    #[deprecated("note on the trait")]
    pub trait Foo {
        #[deprecated]
        fn foo(self);
    }

    pub fn generic<T>(x: T) where T: Foo {
                                     ^^^ note on the trait
                                     ~~~ `Foo` has been deprecated
        x.foo();
        ^^^^^ note on the trait
        ~~~~~ `foo` has been deprecated
    }

    fn main() {}
    "#;
    check_errors(src);
}
