//! Tests for trait method signature matching between trait definitions and implementations.
//! Validates parameter types, return types, parameter counts, and method presence.

use crate::tests::{assert_no_errors, check_errors, check_monomorphization_error};

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
           ~~~~~~~ First trait associated item found here
            y + 2 * x
        }
        // Duplicate trait methods should not compile
        fn default(x: Field, y: Field) -> Field {
           ^^^^^^^ Duplicate definitions of trait associated item with name default found
           ~~~~~~~ Second trait associated item found here
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
