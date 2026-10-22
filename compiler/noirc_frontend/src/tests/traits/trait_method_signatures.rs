//! Tests for trait method signature matching between trait definitions and implementations.
//! Validates parameter types, return types, parameter counts, and method presence.

use crate::elaborator::UnstableFeature;
use crate::tests::{
    assert_no_errors, assert_no_errors_using_features, check_errors, check_errors_with_stdlib,
    check_monomorphization_error,
};

#[test]
fn check_trait_implementation_duplicate_method() {
    let src = "
    trait Default2 {
        fn default(x: Field, y: Field) -> Field;
    }

    struct Foo {
        bar: Field,
        array: [Field; 2],
    }

    impl Default2 for Foo {
        // Duplicate trait methods should not compile
        fn default(x: Field, y: Field) -> Field {
           ~~~~~~~ First definition found here
            y + 2 * x
        }
        // Duplicate trait methods should not compile
        fn default(x: Field, y: Field) -> Field {
           ^^^^^^^ Duplicate definitions of trait associated item with name default found
           ~~~~~~~ Second definition found here
            x + 2 * y
        }
    }

    fn main() {
        let _ = Foo { bar: 1, array: [2, 3] }; // silence Foo never constructed warning
    }";
    check_errors(src);
}

#[test]
fn check_trait_wrong_method_return_type() {
    let src = "
    trait Default2 {
        fn default() -> Self;
    }

    struct Foo {
    }

    impl Default2 for Foo {
        fn default() -> Field {
                        ^^^^^ Expected type Foo, found type Field
            0
        }
    }

    fn main() {
        let _ = Foo {}; // silence Foo never constructed warning
    }
    ";
    check_errors(src);
}

#[test]
fn check_trait_wrong_method_return_type2() {
    let src = "
    trait Default2 {
        fn default(x: Field, y: Field) -> Self;
    }

    struct Foo {
        bar: Field,
        array: [Field; 2],
    }

    impl Default2 for Foo {
        fn default(x: Field, _y: Field) -> Field {
                                           ^^^^^ Expected type Foo, found type Field
            x
        }
    }

    fn main() {
        let _ = Foo { bar: 1, array: [2, 3] }; // silence Foo never constructed warning
    }";
    check_errors(src);
}

#[test]
fn check_trait_wrong_method_return_type3() {
    let src = "
    trait Default2 {
        fn default(x: Field, y: Field) -> Self;
    }

    struct Foo {
        bar: Field,
        array: [Field; 2],
    }

    impl Default2 for Foo {
        fn default(_x: Field, _y: Field) {
                                        ^ Expected type Foo, found type ()
        }
    }

    fn main() {
        let _ = Foo { bar: 1, array: [2, 3] }; // silence Foo never constructed warning
    }
    ";
    check_errors(src);
}

#[test]
fn check_trait_wrong_parameter_type() {
    let src = "
    pub trait Default2 {
        fn default(x: Field, y: NotAType) -> Field;
                                ^^^^^^^^ Could not resolve 'NotAType' in path
    }

    fn main(x: Field, y: Field) {
        assert(y == x);
    }
    ";
    check_errors(src);
}

#[test]
fn returns_self_in_trait_method_3() {
    let src = "
    pub trait MagicNumber {
        fn from_magic_value() -> Self {
            Self::from_value()
        }
        fn from_value() -> Self;
    }

    impl MagicNumber for i32 {
        fn from_value() -> Self {
            0
        }
    }

    impl MagicNumber for i64 {
        fn from_value() -> Self {
            0
        }
    }
    ";
    assert_no_errors(src);
}

#[test]
fn trait_method_numeric_generic_on_function() {
    let src = r#"
    trait Bar {
        fn baz<let N: u32>();
    }

    impl Bar for Field {
        fn baz<let N: u32>() {
            let _ = N;
        }
    }

    fn foo<K: Bar>() {
        K::baz::<2>();
    }

    fn main() {
        foo::<Field>();
    }
    "#;
    check_monomorphization_error(src);
}

#[test]
fn trait_method_numeric_generic_on_function_direct_dispatch() {
    let src = r#"
    trait Bar {
        fn baz<let N: u32>() -> u32;
    }

    impl Bar for Field {
        fn baz<let M: u32>() -> u32 {
            M
        }
    }

    fn main() {
        let _ = <Field as Bar>::baz::<7>();
    }
    "#;
    check_monomorphization_error(src);
}

#[test]
fn check_trait_missing_implementation() {
    let src = "
    trait Default2 {
        fn default(x: Field, y: Field) -> Self;

        fn method2(x: Field) -> Field;

    }

    struct Foo {
        bar: Field,
        array: [Field; 2],
    }

    impl Default2 for Foo {
                      ^^^ Method `method2` from trait `Default2` is not implemented
                      ~~~ Please implement method2 here
        fn default(x: Field, y: Field) -> Self {
            Self { bar: x, array: [x,y] }
        }
    }
    ";
    check_errors(src);
}

#[test]
fn check_trait_wrong_method_name() {
    let src = "
    trait Default2 {
    }

    struct Foo {
        bar: Field,
        array: [Field; 2],
    }

    impl Default2 for Foo {
        fn does_not_exist(x: Field, y: Field) -> Self {
           ^^^^^^^^^^^^^^ Method with name `does_not_exist` is not part of trait `Default2`, therefore it can't be implemented
            Self { bar: x, array: [x,y] }
        }
    }

    fn main() {
        let _ = Foo { bar: 1, array: [2, 3] }; // silence Foo never constructed warning
    }";
    check_errors(src);
}

#[test]
fn check_trait_wrong_parameter() {
    let src = "
    trait Default2 {
        fn default(x: Field) -> Self;
    }

    struct Foo {
        bar: u32,
    }

    impl Default2 for Foo {
        fn default(x: u32) -> Self {
                      ^^^ Parameter #1 of method `default` must be of type Field, not u32
            Foo {bar: x}
        }
    }
    ";
    check_errors(src);
}

#[test]
fn check_trait_wrong_parameter2() {
    let src = "
    trait Default2 {
        fn default(x: Field, y: Field) -> Self;
    }

    struct Foo {
        bar: Field,
        array: [Field; 2],
    }

    impl Default2 for Foo {
        fn default(x: Field, y: Foo) -> Self {
                                ^^^ Parameter #2 of method `default` must be of type Field, not Foo
            Self { bar: x, array: [x, y.bar] }
        }
    }
    ";
    check_errors(src);
}

#[test]
fn check_trait_wrong_parameters_count() {
    let src = "
    trait Default2 {
        fn default(x: Field, y: Field) -> Self;
    }

    struct Foo {
        bar: Field,
        array: [Field; 2],
    }

    impl Default2 for Foo {
        fn default(x: Field) -> Self {
           ^^^^^^^ `Default2::default` expects 2 parameters, but this method has 1
            Self { bar: x, array: [x, x] }
        }
    }
    ";
    check_errors(src);
}

#[test]
fn returns_self_in_trait_method_1() {
    let src = "
    pub trait MagicNumber {
        fn from_magic_value() -> Self;
        fn from_value() -> Self;
    }

    pub struct Foo {}

    impl MagicNumber for Foo {
        fn from_magic_value() -> Foo {
            Self::from_value()
        }
        fn from_value() -> Self {
            Self {}
        }
    }

    pub struct Bar {}

    impl MagicNumber for Bar {
        fn from_magic_value() -> Bar {
            Self::from_value()
        }
        fn from_value() -> Self {
            Self {}
        }
    }
    ";
    assert_no_errors(src);
}

#[test]
fn returns_self_in_trait_method_2() {
    let src = "
    pub trait MagicNumber {
        fn from_magic_value() -> Self {
            Self::from_value()
        }
        fn from_value() -> Self;
    }

    pub struct Foo {}

    impl MagicNumber for Foo {
        fn from_value() -> Self {
            Self {}
        }
    }

    pub struct Bar {}

    impl MagicNumber for Bar {
        fn from_value() -> Self {
            Self {}
        }
    }
    ";
    assert_no_errors(src);
}

#[test]
fn placeholder_not_allowed_in_trait_method_signature() {
    let src = r#"
    pub trait Trait {
        fn trait_method(_: [_; _]) -> [_; _];
                            ^ The placeholder `_` is not allowed in function parameter types
                               ^ The placeholder `_` is not allowed in function parameter types
                                       ^ The placeholder `_` is not allowed in function return types
                                          ^ The placeholder `_` is not allowed in function return types
    }

    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn errors_if_using_comptime_type_in_trait_method_signature() {
    let src = r#"
    pub trait Foo {
        fn foo(_x: Quoted);
                   ^^^^^^ Comptime-only type `Quoted` cannot be used in non-comptime function
    }
    "#;
    check_errors(src);
}

#[test]
fn allows_comptime_type_in_comptime_trait_method_signature() {
    let src = r#"
    pub trait Foo {
        comptime fn foo(_x: Quoted);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn errors_if_using_comptime_type_in_stdlib_trait_method_signature() {
    let src = r#"
    pub trait Foo {
        fn foo(_x: Quoted);
                   ^^^^^^ Comptime-only type `Quoted` cannot be used in non-comptime function
    }
    "#;
    check_errors_with_stdlib(src, []);
}

#[test]
fn allows_comptime_type_in_stdlib_comptime_trait_method_signature() {
    let src = r#"
    pub trait Foo {
        comptime fn foo(_x: Quoted);
    }
    "#;
    check_errors_with_stdlib(src, []);
}

/// https://github.com/noir-lang/noir-claude/issues/2237: the trait method's return type
/// `<Self as Source>::Out` is the impl's `u8`, through the parent bound of its `Self: Checked`.
#[test]
fn impl_method_returning_a_projection_through_a_parent_bound_matches_the_trait() {
    let src = r#"
    trait Source { type Out; fn get(self) -> Self::Out; }
    trait Checked: Source {}
    trait Foo { fn g(self) -> <Self as Source>::Out where Self: Checked; }
    impl Source for u32 { type Out = u8; fn get(self) -> u8 { self as u8 } }
    impl Checked for u32 {}
    impl Foo for u32 { fn g(self) -> <Self as Source>::Out { self.get() } }
    fn main(x: u32) { let r: u8 = x.g(); assert(r == 3); }
    "#;
    assert_no_errors(src);
}

/// https://github.com/noir-lang/noir-claude/issues/2752: an impl of a trait with a `where Self:`
/// bound doesn't name the bound's associated type.
#[test]
fn trait_with_a_where_self_bound_eliding_an_associated_type_can_be_implemented_without_methods() {
    let src = r#"
    trait Lim { type Out; }
    struct A {}
    impl Lim for A { type Out = u8; }
    trait Chk where Self: Lim { fn chk() {} }
    impl Chk for A {}
    fn main() { A::chk(); }
    "#;
    assert_no_errors(src);
}

/// https://github.com/noir-lang/noir-claude/issues/2752: `where Self: Lim` leaves `Lim`'s
/// associated type out, as a supertrait `trait Chk: Lim` does.
#[test]
fn trait_with_a_where_self_bound_eliding_an_associated_type_can_be_implemented() {
    let src = r#"
    trait Lim { type Out; fn get(self) -> Self::Out; }
    struct A { v: u8 }
    impl Lim for A { type Out = u8; fn get(self) -> u8 { self.v } }
    trait Chk where Self: Lim { fn chk(self) -> <Self as Lim>::Out { self.get() } }
    impl Chk for A {}
    fn main(x: u8) -> pub u8 { A { v: x }.chk() }
    "#;
    assert_no_errors(src);
}

/// https://github.com/noir-lang/noir-claude/issues/2049: the trait method's `impl Bar` parameter
/// stands for the impl method's own.
#[test]
fn trait_method_with_an_impl_trait_parameter_can_be_implemented() {
    let src = r#"
    trait Bar { fn get(self) -> Field; }
    impl Bar for Field { fn get(self) -> Field { self + 1 } }
    trait Tr { fn f(a: impl Bar) -> Field; }
    struct S {}
    impl Tr for S { fn f(a: impl Bar) -> Field { a.get() } }
    fn main(x: Field) -> pub Field { S::f(x) }
    "#;
    assert_no_errors_using_features(src, &[UnstableFeature::TraitAsType]);
}

/// https://github.com/noir-lang/noir-claude/issues/1711: a numeric generic can't stand for the
/// trait method's type generic.
#[test]
fn impl_method_generic_of_another_kind_than_the_traits_is_rejected() {
    let src = r#"
    trait Marker {}
    trait MyT { fn f<T>(self, x: T) where T: Marker; }
    pub struct B { x: Field }
    impl MyT for B { fn f<let Q: u32>(_self: Self, _x: Field) where Q: Marker {} }
                                                                    ^ Expected type, found numeric generic
                                                                    ~ not a type
                              ^^^^^^ Type provided when a numeric generic was expected
                              ~~~~~~ the numeric generic is not of type `u32`
                        ^ Expected type fn(B, _) -> (), found type fn(B, Field) -> ()
    fn main() {}
    "#;
    check_errors(src);
}
