//! Tests for trait implementation validation.
//! Validates duplicate impls, impl target correctness, missing associated items, and generic counts.

use crate::tests::check_errors;

#[test]
fn check_trait_impl_for_non_type() {
    let src = "
    trait Default2 {
        fn default(x: Field, y: Field) -> Field;
    }

    impl Default2 for main {
                      ^^^^ expected type, found function `main`
        fn default(x: Field, y: Field) -> Field {
            x + y
        }
    }

    fn main() {}
    ";
    check_errors(src);
}

#[test]
fn check_trait_duplicate_implementation() {
    let src = "
    trait Default2 {
    }
    struct Foo {
        bar: Field,
    }

    impl Default2 for Foo {
         ~~~~~~~~ Previous impl defined here
    }
    impl Default2 for Foo {
                      ^^^ Impl for type `Foo` overlaps with existing impl
                      ~~~ Overlapping impl
    }
    fn main() {
        let _ = Foo { bar: 1 }; // silence Foo never constructed warning
    }
    ";
    check_errors(src);
}

#[test]
fn check_trait_duplicate_implementation_with_alias() {
    let src = "
    trait Default2 {
    }

    struct MyStruct {
    }

    type MyType = MyStruct;

    impl Default2 for MyStruct {
         ~~~~~~~~ Previous impl defined here
    }

    impl Default2 for MyType {
                      ^^^^^^ Impl for type `MyType` overlaps with existing impl
                      ~~~~~~ Overlapping impl
    }

    fn main() {
        let _ = MyStruct {}; // silence MyStruct never constructed warning
    }
    ";
    check_errors(src);
}

#[test]
fn error_on_duplicate_impl_with_associated_type() {
    let src = r#"
        trait Foo {
            type Bar;
        }

        impl Foo for i32 {
             ~~~ Previous impl defined here
            type Bar = u32;
        }

        impl Foo for i32 {
                     ^^^ Impl for type `i32` overlaps with existing impl
                     ~~~ Overlapping impl
            type Bar = u8;
        }
    "#;
    check_errors(src);
}

#[test]
fn error_on_duplicate_impl_with_associated_constant() {
    let src = r#"
        trait Foo {
            let Bar: u32;
        }

        impl Foo for i32 {
             ~~~ Previous impl defined here
            let Bar: u32 = 5;
        }

        impl Foo for i32 {
                     ^^^ Impl for type `i32` overlaps with existing impl
                     ~~~ Overlapping impl
            let Bar: u32 = 6;
        }
    "#;
    check_errors(src);
}

#[test]
fn impl_missing_associated_type() {
    let src = r#"
    trait Foo {
        type Assoc;
    }

    impl Foo for () {}
         ^^^ `Foo` is missing the associated type `Assoc`
    "#;
    check_errors(src);
}

#[test]
fn unconstrained_type_parameter_in_trait_impl() {
    let src = r#"
        pub trait Trait<T> {}
        pub struct Foo<T> {}

        impl<T, U> Trait<T> for Foo<T> {}
                ^ The type parameter `U` is not constrained by the impl trait, self type, or predicates
                ~ Hint: remove the `U` type parameter
        "#;
    check_errors(src);
}

#[test]
fn trait_impl_generics_count_mismatch() {
    let src = r#"
    trait Foo {}

    impl Foo<()> for Field {}
         ^^^ Foo expects 0 generics but 1 was given
    "#;
    check_errors(src);
}

#[test]
fn errors_if_constrained_trait_definition_has_unconstrained_impl() {
    let src = r#"
    pub trait Foo {
        fn foo() -> Field;
    }

    impl Foo for Field {
        unconstrained fn foo() -> Field {
                         ^^^ foo is not expected to be unconstrained
            42
        }
    }
    "#;
    check_errors(src);
}

#[test]
fn errors_if_unconstrained_trait_definition_has_constrained_impl() {
    let src = r#"
    pub trait Foo {
        unconstrained fn foo() -> Field;
    }

    impl Foo for Field {
        fn foo() -> Field {
           ^^^ foo is expected to be unconstrained
            42
        }
    }
    "#;
    check_errors(src);
}

#[test]
fn check_impl_struct_not_trait() {
    let src = "
    struct Foo {
        bar: Field,
        array: [Field; 2],
    }

    struct Default2 {
        x: Field,
        z: Field,
    }

    impl Default2 for Foo {
         ^^^^^^^^ Default2 is not a trait, therefore it can't be implemented
        fn default(x: Field, y: Field) -> Self {
            Self { bar: x, array: [x,y] }
        }
    }

    fn main() {
        let _ = Default2 { x: 1, z: 1 }; // silence Default2 never constructed warning
    }
    ";
    check_errors(src);
}

#[test]
fn check_trait_duplicate_declaration() {
    let src = "
    trait Default2 {
          ~~~~~~~~ First trait definition found here
        fn default(x: Field, y: Field) -> Self;
    }

    struct Foo {
        bar: Field,
        array: [Field; 2],
    }

    impl Default2 for Foo {
        fn default(x: Field,y: Field) -> Self {
            Self { bar: x, array: [x,y] }
        }
    }

    trait Default2 {
          ^^^^^^^^ Duplicate definitions of trait definition with name Default2 found
          ~~~~~~~~ Second trait definition found here
        fn default(x: Field) -> Self;
    }
    ";
    check_errors(src);
}

#[test]
fn check_trait_not_in_scope() {
    let src = "
    struct Foo {
        bar: Field,
        array: [Field; 2],
    }

    impl Default2 for Foo {
         ^^^^^^^^ Trait Default2 not found
        fn default(x: Field, y: Field) -> Self {
            Self { bar: x, array: [x,y] }
        }
    }
    ";
    check_errors(src);
}

#[test]
fn trait_impl_associated_type_without_body() {
    let src = "
    pub trait Trait {
        type Assoc;
    }

    impl Trait for Field {
        type Assoc;
             ^^^^^ Associated type in impl without body
             ~~~~~ Provide a definition for the type: ` = <type>;`
    }

    fn main() {}
    ";
    check_errors(src);
}
