//! Tests for "impl stricter than trait" validation.
//! Ensures that trait implementations don't add constraints that aren't present in the trait definition.

use crate::tests::check_errors;

#[test]
fn impl_stricter_than_trait_no_trait_method_constraints() {
    // This test ensures that the error we get from the where clause on the trait impl method
    // is a `DefCollectorErrorKind::ImplIsStricterThanTrait` error.
    let src = r#"
    trait Serialize<let N: u32> {
        // We want to make sure we trigger the error when override a trait method
        // which itself has no trait constraints.
        fn serialize(self) -> [Field; N];
           ~~~~~~~~~ definition of `serialize` from trait
    }

    trait ToField {
        fn to_field(self) -> Field;
    }

    fn process_array<let N: u32>(array: [Field; N]) -> Field {
        array[0]
    }

    fn serialize_thing<A, let N: u32>(thing: A) -> [Field; N] where A: Serialize<N> {
        thing.serialize()
    }

    struct MyType<T> {
        a: T,
        b: T,
    }

    impl<T> Serialize<2> for MyType<T> {
        fn serialize(self) -> [Field; 2] where T: ToField {
                                                  ^^^^^^^ impl has stricter requirements than trait
                                                  ~~~~~~~ impl has extra requirement `T: ToField`
            [ self.a.to_field(), self.b.to_field() ]
        }
    }

    impl<T> MyType<T> {
        fn do_thing_with_serialization_with_extra_steps(self) -> Field {
            process_array(serialize_thing(self))
        }
    }

    fn main() {
        let _ = MyType { a: 1, b: 1 }; // silence MyType never constructed warning
    }
    "#;
    check_errors(src);
}

#[test]
fn impl_stricter_than_trait_different_generics() {
    let src = r#"
    trait Default2 { }

    // Object type of the trait constraint differs
    trait Foo<T> {
        fn foo_good<U>() where T: Default2;

        fn foo_bad<U>() where T: Default2;
           ~~~~~~~ definition of `foo_bad` from trait
    }

    impl<A> Foo<A> for () {
        fn foo_good<B>() where A: Default2 {}

        fn foo_bad<B>() where B: Default2 {}
                                 ^^^^^^^^ impl has stricter requirements than trait
                                 ~~~~~~~~ impl has extra requirement `B: Default2`
    }
    "#;
    check_errors(src);
}

#[test]
fn impl_stricter_than_trait_different_trait() {
    let src = r#"
    trait Default2 { }

    trait OtherDefault { }

    struct Option2<T> {
        inner: T
    }

    trait Bar<T> {
        fn bar<U>() where Option2<T>: Default2;
           ~~~ definition of `bar` from trait
    }

    impl<A> Bar<A> for () {
        // Trait constraint differs due to the trait even though the constraint
        // types are the same.
        fn bar<B>() where Option2<A>: OtherDefault {}
                                      ^^^^^^^^^^^^ impl has stricter requirements than trait
                                      ~~~~~~~~~~~~ impl has extra requirement `Option2<A>: OtherDefault`
    }

    fn main() {
        let _ = Option2 { inner: 1 }; // silence Option2 never constructed warning
    }
    "#;
    check_errors(src);
}

#[test]
fn trait_impl_where_clause_stricter_pass() {
    let src = r#"
    trait MyTrait {
        fn good_foo<T, H>() where H: OtherTrait;

        fn bad_foo<T, H>() where H: OtherTrait;
           ~~~~~~~ definition of `bad_foo` from trait
    }

    trait OtherTrait {}

    struct Option2<T> {
        inner: T
    }

    impl<T> MyTrait for [T] where Option2<T>: MyTrait {
        fn good_foo<A, B>() where B: OtherTrait { }

        fn bad_foo<A, B>() where A: OtherTrait { }
                                    ^^^^^^^^^^ impl has stricter requirements than trait
                                    ~~~~~~~~~~ impl has extra requirement `A: OtherTrait`
    }

    fn main() {
        let _ = Option2 { inner: 1 }; // silence Option2 never constructed warning
    }
    "#;
    check_errors(src);
}

#[test]
fn impl_stricter_than_trait_different_trait_generics() {
    let src = r#"
    trait Foo<T> {
        fn foo<U>() where T: T2<T>;
           ~~~ definition of `foo` from trait
    }

    impl<A> Foo<A> for () {
        // Should be A: T2<A>
        fn foo<B>() where A: T2<B> {}
                             ^^ impl has stricter requirements than trait
                             ~~ impl has extra requirement `A: T2<B>`
    }

    trait T2<C> {}
    "#;
    check_errors(src);
}

#[test]
fn impl_stricter_than_trait_different_object_generics() {
    let src = r#"
    trait MyTrait { }

    trait OtherTrait {}

    struct Option2<T> {
        inner: T
    }

    struct OtherOption<T> {
        inner: Option2<T>,
    }

    trait Bar<T> {
        fn bar_good<U>() where Option2<T>: MyTrait, OtherOption<Option2<T>>: OtherTrait;

        fn bar_bad<U>() where Option2<T>: MyTrait, OtherOption<Option2<T>>: OtherTrait;
           ~~~~~~~ definition of `bar_bad` from trait

        fn array_good<U>() where [T; 8]: MyTrait;

        fn array_bad<U>() where [T; 8]: MyTrait;
           ~~~~~~~~~ definition of `array_bad` from trait

        fn tuple_good<U>() where (Option2<T>, Option2<U>): MyTrait;

        fn tuple_bad<U>() where (Option2<T>, Option2<U>): MyTrait;
           ~~~~~~~~~ definition of `tuple_bad` from trait
    }

    impl<A> Bar<A> for () {
        fn bar_good<B>()
        where
            OtherOption<Option2<A>>: OtherTrait,
            Option2<A>: MyTrait { }

        fn bar_bad<B>()
        where
            OtherOption<Option2<A>>: OtherTrait,
            Option2<B>: MyTrait { }
                        ^^^^^^^ impl has stricter requirements than trait
                        ~~~~~~~ impl has extra requirement `Option2<B>: MyTrait`

        fn array_good<B>() where [A; 8]: MyTrait { }

        fn array_bad<B>() where [B; 8]: MyTrait { }
                                        ^^^^^^^ impl has stricter requirements than trait
                                        ~~~~~~~ impl has extra requirement `[B; 8]: MyTrait`

        fn tuple_good<B>() where (Option2<A>, Option2<B>): MyTrait { }

        fn tuple_bad<B>() where (Option2<B>, Option2<A>): MyTrait { }
                                                          ^^^^^^^ impl has stricter requirements than trait
                                                          ~~~~~~~ impl has extra requirement `(Option2<B>, Option2<A>): MyTrait`
    }

    fn main() {
        let _ = OtherOption { inner: Option2 { inner: 1 } }; // silence unused warnings
    }
    "#;
    check_errors(src);
}
