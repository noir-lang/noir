//! Tests for associated types and associated constants in traits.
//! Validates accessing, computing with, and constraining associated items.

use crate::tests::{assert_no_errors, check_errors, check_monomorphization_error};

#[test]
fn passes_trait_with_associated_number_to_generic_function() {
    let src = "
    trait Trait {
        let N: u32;
    }

    pub struct Foo {}

    impl Trait for Foo {
        let N: u32 = 1;
    }

    fn main() {
        foo::<Foo>();
    }

    fn foo<T>()
    where
        T: Trait,
    {}
    ";
    assert_no_errors(src);
}

#[test]
fn passes_trait_with_associated_number_to_generic_function_inside_struct_impl() {
    let src = "
    trait Trait {
        let N: u32;
    }

    pub struct Foo {}

    impl Trait for Foo {
        let N: u32 = 1;
    }

    pub struct Bar<T> {}

    impl<T> Bar<T> {
        fn bar<U>(self) where U: Trait {
            let _ = self;
        }
    }

    fn main() {
        let bar = Bar::<i32> {};
        bar.bar::<Foo>();
    }
    ";
    assert_no_errors(src);
}

#[test]
fn accesses_associated_type_inside_trait_impl_using_self() {
    let src = r#"
    pub trait Trait {
        let N: u32;

        fn foo() -> u32;
    }

    impl Trait for i32 {
        let N: u32 = 10;

        fn foo() -> u32 {
            Self::N
        }
    }

    fn main() {
        let _ = i32::foo();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn accesses_associated_type_inside_trait_using_self() {
    let src = r#"
    pub trait Trait {
        let N: u32;

        fn foo() -> u32 {
            Self::N
        }
    }

    impl Trait for i32 {
        let N: u32 = 10;
    }

    fn main() {
        let _ = i32::foo();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn serialize_test_with_a_previous_unrelated_definition() {
    let src = r#"
    // There used to be a bug where this unrelated definition would cause compilation to fail
    // with a "No impl found" error.
    pub trait Trait {}

    trait Serialize {
        let Size: u32;

        fn serialize(self);
    }

    impl<A, B> Serialize for (A, B)
    where
        A: Serialize,
        B: Serialize,
    {
        let Size: u32 = <A as Serialize>::Size + <B as Serialize>::Size;

        fn serialize(self: Self) {
            self.0.serialize();
        }
    }

    impl Serialize for Field {
        let Size: u32 = 1;

        fn serialize(self) { }
    }

    fn main() {
        let x = (((1, 2), 5), 9);
        x.serialize();
    }
    "#;
    check_monomorphization_error(src);
}

#[test]
fn associated_constant_of_generic_type_used_in_another_associated_constant() {
    let src = r#"
    trait Serialize {
        let N: u32;

        fn serialize(self) -> [Field; N];
    }

    impl<let M: u32> Serialize for [Field; M] {
        let N: u32 = M;

        fn serialize(self) -> [Field; Self::N] {
            self
        }
    }

    struct Foo {}

    impl Serialize for Foo {
        let N: u32 = <[Field; 3] as Serialize>::N;

        fn serialize(self) -> [Field; Self::N] {
            [0; Self::N]
        }
    }

    fn main() {
        let _ = Foo {}.serialize();
    }
    "#;
    check_monomorphization_error(src);
}

#[test]
fn associated_constant_of_generic_type_used_in_expression() {
    let src = r#"
    trait Serialize {
        let N: u32;
    }

    impl<let M: u32> Serialize for [Field; M] {
        let N: u32 = M;
    }

    fn main() {
        let _ = <[Field; 3] as Serialize>::N;
    }
    "#;
    check_monomorphization_error(src);
}

#[test]
fn ambiguous_associated_type() {
    let src = r#"
    trait MyTrait {
        type X;
    }

    fn main() {
        let _: MyTrait::X = 1;
               ^^^^^^^^^^ Ambiguous associated type
               ~~~~~~~~~~ If there were a type named `Example` that implemented `MyTrait`, you could use the fully-qualified path: `<Example as MyTrait>::X`
    }
    "#;
    check_errors(src);
}

#[test]
fn associated_constant_sum_of_other_constants() {
    let src = r#"
    pub trait Deserialize {
        let N: u32;

        fn deserialize(_: [Field; Self::N]);
    }

    impl Deserialize for Field {
        let N: u32 = 1;

        fn deserialize(_: [Field; Self::N]) {}
    }

    struct Gen<T> {}

    impl<T> Deserialize for Gen<T>
    where
        T: Deserialize,
    {
        let N: u32 = <T as Deserialize>::N + <T as Deserialize>::N;

        fn deserialize(_: [Field; Self::N]) {}
    }

    fn main() {
        let f = <Gen<Field> as Deserialize>::deserialize;
        f([0; 2]);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn associated_constant_sum_of_other_constants_2() {
    let src = r#"
    pub trait Deserialize {
        let N: u32;

        fn deserialize(_: [Field; N]);
    }

    impl Deserialize for Field {
        let N: u32 = 1;

        fn deserialize(_: [Field; Self::N]) {}
    }

    impl<T, let M: u32> Deserialize for [T; M]
    where
        T: Deserialize,
    {
        let N: u32 = <T as Deserialize>::N + M;

        fn deserialize(_: [Field; Self::N]) {}
    }

    pub fn foo<let X: u32>() {
        let f = <[Field; X] as Deserialize>::deserialize;
        let _ = f([0; X + 1]);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn associated_constant_sum_of_other_constants_3() {
    let src = r#"
    pub trait Deserialize {
        let N: u32;

        fn deserialize(_: [Field; N]);
    }

    impl Deserialize for Field {
        let N: u32 = 1;

        fn deserialize(_: [Field; Self::N]) {}
    }

    impl<T, let M: u32> Deserialize for [T; M]
    where
        T: Deserialize,
    {
        let N: u32 = <T as Deserialize>::N + M - 1;

        fn deserialize(_: [Field; Self::N]) {}
    }

    pub fn foo<let X: u32>() {
        let f = <[Field; X] as Deserialize>::deserialize;
        let _ = f([0; X]);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn associated_constant_mul_of_other_constants() {
    let src = r#"
    pub trait Deserialize {
        let N: u32;

        fn deserialize(_: [Field; N]);
    }

    impl Deserialize for Field {
        let N: u32 = 1;

        fn deserialize(_: [Field; Self::N]) {}
    }

    impl<T, let M: u32> Deserialize for [T; M]
    where
        T: Deserialize,
    {
        let N: u32 = <T as Deserialize>::N * M;

        fn deserialize(_: [Field; Self::N]) {}
    }

    pub fn foo<let X: u32>() {
        let f = <[Field; X] as Deserialize>::deserialize;
        let _ = f([0; X]);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_impl_with_where_clause_with_trait_with_associated_numeric() {
    let src = "
    trait Bar {
        let N: Field;
    }

    impl Bar for Field {
        let N: Field = 42;
    }

    trait Foo {
        fn foo<B>(b: B) where B: Bar; 
    }

    impl Foo for Field{
        fn foo<B>(_: B) where B: Bar {} 
    }
    ";
    assert_no_errors(src);
}

#[test]
fn trait_impl_with_where_clause_with_trait_with_associated_type() {
    let src = "
    trait Bar {
        type typ;
    }

    impl Bar for Field {
        type typ = Field;
    }

    trait Foo {
        fn foo<B>(b: B) where B: Bar; 
    }

    impl Foo for Field{
        fn foo<B>(_: B) where B: Bar {} 
    }
    ";
    assert_no_errors(src);
}
