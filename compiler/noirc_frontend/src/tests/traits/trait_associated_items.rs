//! Tests for associated types and associated constants in traits.
//! Validates accessing, computing with, and constraining associated items.

use crate::tests::{assert_no_errors, check_errors};

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
fn accesses_associated_constant_inside_trait_impl_using_self() {
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
fn accesses_associated_constant_inside_trait_using_self() {
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
fn accesses_associated_type_inside_trait_and_impl_using_self() {
    let src = r#"
    pub struct CustomType {}

    pub trait Trait {
        type Output;
        fn foo() -> Self::Output;
    }

    impl Trait for i32 {
        type Output = CustomType;

        fn foo() -> Self::Output {
            CustomType {}
        }
    }

    fn main() {
        let _: CustomType = i32::foo();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn accesses_associated_constant_on_data_type_using_self() {
    let src = r#"
    trait Container {
        let N: u32;
        fn get_item() -> u32;
    }

    struct MyContainer {}

    impl Container for MyContainer {
        let N: u32 = 10;

        fn get_item() -> u32 {
            Self::N
        }
    }

    fn main() {
        let _: u32 = MyContainer::get_item();
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
    assert_no_errors(src);
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
    assert_no_errors(src);
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
    assert_no_errors(src);
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

#[test]
fn self_associated_constant_from_different_trait() {
    // Self::N resolves based on which trait impl we're in, even when multiple traits define a constant with the same name
    let src = r#"
    trait Trait1 {
        let N: u32;
    }

    trait Trait2 {
        let N: u32;
        fn get_n() -> u32;
    }

    impl Trait1 for u32 {
        let N: u32 = 100;
    }

    impl Trait2 for u32 {
        let N: u32 = 200;
        fn get_n() -> u32 {
            // Self::N should resolve to Trait2's N (200), not Trait1's N (100)
            Self::N
        }
    }

    fn main() {
        assert(u32::get_n() == 200);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn self_associated_constant_does_not_cross_trait_boundaries() {
    // Self::AssociatedConstant cannot access constants from other trait impls
    let src = r#"
    trait Base {
        let N: u32;
    }

    trait Derived {
        fn get_base() -> u32;
    }

    impl Base for u32 {
        let N: u32 = 10;
    }

    impl Derived for u32 {
        fn get_base() -> u32 {
            Self::N
                  ^ No method named 'N' found for type 'u32'
        }
    }

    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn associated_and_generic_type_share_name() {
    let src = r#"
    pub trait Foo<Bar> {
        type Bar;

        fn gen_to_assoc(x: Bar) -> Self::Bar {
                                   ^^^^^^^^^ expected type Self::Bar, found type Bar
                                   ~~~~~~~~~ expected Self::Bar because of return type
            x
            ~ Bar returned here
        }

        fn assoc_to_gen(x: Self::Bar) -> Bar {
                                         ^^^ expected type Bar, found type Self::Bar
                                         ~~~ expected Bar because of return type
            x
            ~ Self::Bar returned here
        }
    }
    "#;
    check_errors(src);
}

#[test]
fn associated_type_mismatch_across_traits() {
    let src = r#"
    pub trait Spam {
        type Item;
        fn give_spam() -> Self::Item;
    }

    pub trait Eggs {
        type Item;
        fn take_eggs(eggs: Self::Item);
    }

    pub fn mix<A: Spam, B: Eggs>() {
        B::take_eggs(A::give_spam());
                     ^^^^^^^^^^^^^^ Expected type <B as Eggs>::Item, found type <A as Spam>::Item
    }
    "#;
    check_errors(src);
}

#[test]
fn associated_type_mismatch_across_modules() {
    // Error message is confusing here but it is an improvement over no error
    let src = r#"
        pub mod one {
            pub trait Eggs {
                type Item;
                fn give() -> Self::Item;
            }
        }

        pub mod two {
            pub trait Eggs {
                type Item;
                fn take(eggs: Self::Item);
            }
        }

        pub fn mix<T: one::Eggs + two::Eggs>() {
            T::take(T::give());
                    ^^^^^^^^^ Expected type <T as Eggs>::Item, found type <T as Eggs>::Item
        }

        fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn associated_type_behind_self_as_trait() {
    let src = r#"
    pub trait Foo {
        type Bar;
        fn bar_one() -> Self::Bar;
        fn bar_two() -> <Self as Foo>::Bar;
    }
    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn associated_type_behind_self_as_trait_with_generics() {
    let src = r#"
    pub trait Foo<Baz> {
        type Bar;
        fn bar_one() -> Self::Bar;
        fn bar_two() -> <Self as Foo<Baz>>::Bar;
    }
    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn associated_constant_direct_access() {
    let src = "
    trait MyTrait {
        let N: u32;
    }
    struct Foo {}
    impl MyTrait for Foo {
        let N: u32 = 5;
    }
    fn main() {
        let _: u32 = Foo::N;
    }
    ";
    assert_no_errors(src);
}

/// TODO(https://github.com/noir-lang/noir/issues/11362): Improve error message for missing associated constants
#[test]
fn associated_constant_direct_access_no_impl() {
    let src = r#"
    trait MyTrait {
        let N: u32;
    }
    struct Foo {}
    struct Bar {}
    impl MyTrait for Bar {
        let N: u32 = 5;
    }
    fn main() {
        let _ = Bar {};
        let _: u32 = Foo::N;
                          ^ Could not resolve 'N' in path
    }
    "#;
    check_errors(src);
}

#[test]
fn associated_constant_direct_access_generic_impl() {
    // Verify that Foo::N works when the impl is generic.
    // The impl is for Wrapper<T>, and we access Wrapper<Field>::N.
    // This requires unification to match Wrapper<Field> against Wrapper<T>.
    let src = "
    trait MyTrait {
        let N: u32;
    }
    struct Wrapper<T> { inner: T }
    impl<T> MyTrait for Wrapper<T> {
        let N: u32 = 10;
    }
    fn main() {
        let _: u32 = Wrapper::<Field>::N;
    }
    ";
    assert_no_errors(src);
}

#[test]
fn associated_constant_direct_access_generic_impl_wrong_struct() {
    // Verify that unification correctly rejects non-matching struct types.
    // We have impl MyTrait for Wrapper<T>, but try to access Other<Field>::N.
    // Unification should NOT match Wrapper<T> with Other<Field>.
    let src = r#"
    trait MyTrait {
        let N: u32;
    }
    struct Wrapper<T> { inner: T }
    struct Other<T> { inner: T }
    impl<T> MyTrait for Wrapper<T> {
        let N: u32 = 10;
    }
    fn main() {
        let _ = Wrapper::<Field> { inner: 1 };
        let _ = Other::<Field> { inner: 1 };
        let _: u32 = Other::<Field>::N;
                                     ^ Could not resolve 'N' in path
    }
    "#;
    check_errors(src);
}

#[test]
fn associated_constant_direct_access_generic_impl_wrong_type_arg() {
    // Verify that unification correctly distinguishes between
    // different concrete instantiations of the same generic type.
    // We have impl MyTrait for Wrapper<Field>, but try to access Wrapper<u32>::N.
    // Unification should NOT match Wrapper<Field> with Wrapper<u32>.
    let src = r#"
    trait MyTrait {
        let N: u32;
    }
    struct Wrapper<T> { inner: T }
    impl MyTrait for Wrapper<Field> {
        let N: u32 = 10;
    }
    fn main() {
        let _ = Wrapper::<Field> { inner: 1 };
        let _ = Wrapper::<u32> { inner: 1 };
        let _: u32 = Wrapper::<u32>::N;
                                     ^ Could not resolve 'N' in path
    }
    "#;
    check_errors(src);
}

#[test]
fn associated_constant_direct_access_ambiguous() {
    let src = r#"
    trait Trait1 {
        let N: u32;
    }
    trait Trait2 {
        let N: u32;
    }
    struct Bar {}
    impl Trait1 for Bar {
        let N: u32 = 1;
    }
    impl Trait2 for Bar {
        let N: u32 = 2;
    }
    fn main() {
        let _ = Bar::N;
                     ^ Multiple applicable items in scope
                     ~ Multiple traits which provide `N` are implemented and in scope: `Trait1`, `Trait2`
    }
    "#;
    check_errors(src);
}

#[test]
fn associated_constant_direct_access_ambiguous_resolved_with_fully_qualified_path() {
    let src = "
    trait Trait1 {
        let N: u32;
    }
    trait Trait2 {
        let N: u32;
    }
    struct Bar {}
    impl Trait1 for Bar {
        let N: u32 = 1;
    }
    impl Trait2 for Bar {
        let N: u32 = 2;
    }
    fn main() {
        let _: u32 = <Bar as Trait1>::N;
        let _: u32 = <Bar as Trait2>::N;
    }
    ";
    assert_no_errors(src);
}

// TODO(https://github.com/noir-lang/noir/issues/10770): Improve error message for Foo::MyType syntax for associated types
#[test]
fn associated_type_direct_access() {
    let src = r#"
    pub struct CustomType {}

    trait MyTrait {
        type MyType;
    }
    struct Foo {}
    impl MyTrait for Foo {
        type MyType = CustomType;
    }
    fn main() {
        // Succeeds
        // let _: <Foo as MyTrait>::MyType = CustomType { };
        // Fails
        let _: Foo::MyType = CustomType { };
                    ^^^^^^ Could not resolve 'MyType' in path
    }"#;
    check_errors(src);
}
