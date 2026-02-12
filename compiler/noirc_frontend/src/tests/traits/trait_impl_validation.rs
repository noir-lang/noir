//! Tests for trait implementation validation.
//! Validates duplicate impls, impl target correctness, missing associated items, and generic counts.

use crate::tests::{assert_no_errors, check_errors};

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

#[test]
fn trait_impl_overlap() {
    let src = r#"
    trait Trait { }

    impl<T> Trait for T { }
            ~~~~~ Previous impl defined here
    impl Trait for u32 { }
                   ^^^ Impl for type `u32` overlaps with existing impl
                   ~~~ Overlapping impl
    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn regression_6581_impl_only() {
    let src = "
    trait Foo {
        fn foo(self) -> Self;
    }

    impl<T, let N: u32> Foo for () {
         ^ The type parameter `T` is not constrained by the impl trait, self type, or predicates
         ~ Hint: remove the `T` type parameter
                ^ The type parameter `N` is not constrained by the impl trait, self type, or predicates
                ~ Hint: remove the `N` type parameter
        fn foo(self) -> Self {
            ()
        }
    }
    ";
    check_errors(src);
}

#[test]
fn regression_6581_using_impl_method() {
    let src = "
    trait Foo {
        fn foo(self) -> Self;
    }

    impl<T, let N: u32> Foo for () {
         ^ The type parameter `T` is not constrained by the impl trait, self type, or predicates
         ~ Hint: remove the `T` type parameter
                ^ The type parameter `N` is not constrained by the impl trait, self type, or predicates
                ~ Hint: remove the `N` type parameter
        fn foo(self) -> Self {
            ()
        }
    }

    fn println<T>(_x: T) {}

    fn main() {
        println(().foo());
    }
    ";
    check_errors(src);
}

#[test]
fn multiple_trait_impls_different_type_params() {
    let src = r#"
    struct Container<T> {
        value: T,
    }

    trait Convert {
        fn convert(self) -> Field;
    }

    impl Convert for Container<Field> {
        fn convert(self) -> Field {
            self.value
        }
    }

    impl Convert for Container<bool> {
        fn convert(self) -> Field {
            if self.value { 1 } else { 0 }
        }
    }

    fn main() {
        let c1 = Container { value: 42 as Field };
        let c2 = Container { value: true };
        let _ = c1.convert();
        let _ = c2.convert();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn multiple_trait_impls_numeric_and_type_params() {
    let src = r#"
    struct Buffer<let N: u32, T> {
        data: [T; N],
    }

    trait Sum {
        fn sum(self) -> Field;
    }

    impl<let N: u32> Sum for Buffer<N, Field> {
        fn sum(self) -> Field {
            let mut s: Field = 0;
            for i in 0..N {
                s += self.data[i];
            }
            s
        }
    }

    impl<let N: u32> Sum for Buffer<N, bool> {
        fn sum(self) -> Field {
            let mut s: Field = 0;
            for i in 0..N {
                if self.data[i] { s += 1; }
            }
            s
        }
    }

    fn main() {
        let b1 = Buffer { data: [1, 2, 3] };
        let b2 = Buffer { data: [true, false, true] };
        assert(b1.sum() == 6);
        assert(b2.sum() == 2);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn specialized_impls_for_different_concrete_types() {
    let src = r#"
    trait Describe {
        fn describe(self) -> Field;
    }

    struct Tagged<T> {
        tag: Field,
        inner: T,
    }

    impl Describe for Tagged<Field> {
        fn describe(self) -> Field {
            self.tag + self.inner
        }
    }

    impl Describe for Tagged<bool> {
        fn describe(self) -> Field {
            if self.inner { self.tag } else { 0 }
        }
    }

    impl Describe for Tagged<u32> {
        fn describe(self) -> Field {
            self.tag + self.inner as Field
        }
    }

    fn main() {
        let t1 = Tagged { tag: 1, inner: 2 as Field };
        let t2 = Tagged { tag: 3, inner: true };
        let t3 = Tagged { tag: 4, inner: 5 as u32 };
        assert(t1.describe() == 3);
        assert(t2.describe() == 3);
        assert(t3.describe() == 9);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn generic_struct_implementing_generic_trait() {
    let src = r#"
    trait Convert<U> {
        fn convert(self) -> U;
    }

    struct Wrapper<T> {
        val: T,
    }

    impl Convert<Field> for Wrapper<u32> {
        fn convert(self) -> Field {
            self.val as Field
        }
    }

    impl Convert<Field> for Wrapper<bool> {
        fn convert(self) -> Field {
            if self.val { 1 } else { 0 }
        }
    }

    fn to_field<T>(w: T) -> Field where T: Convert<Field> {
        w.convert()
    }

    fn main() {
        let w1 = Wrapper { val: 42 as u32 };
        let w2 = Wrapper { val: true };
        assert(to_field(w1) == 42);
        assert(to_field(w2) == 1);
    }
    "#;
    assert_no_errors(src);
}
