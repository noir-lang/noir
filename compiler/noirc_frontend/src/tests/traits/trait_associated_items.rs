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
fn associated_type_mismatch_with_inheritance() {
    // This code would be rejected by Rust, without further evidence to support their equivalence.
    let src = r#"
    pub trait Foo {
        type Bar;
        fn foo(x: Self::Bar) -> Self::Bar;
    }

    pub trait Qux: Foo {
        type Baz;
        fn qux(x: Self::Baz) -> Self::Baz {
            <Self as Foo>::foo(x)
             ^^^^^^^^^^^ No matching impl found for `Self: Foo<Bar = Self::Baz>`
             ~~~~~~~~~~~ No impl for `Self: Foo<Bar = Self::Baz>`
        }
    }

    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn associated_type_and_constant_composite() {
    let src = r#"
    pub trait Foo {
        type Bar;
        let Baz: u32;
        fn foo(x: [Self::Bar; Self::Baz]) -> u32;
    }

    impl Foo for () {
        type Bar = ();
        let Baz: u32 = 0;
        fn foo(_x: [Self::Bar; Self::Baz]) -> u32 {
            0
        }
    }

    fn main() {
        let _ = <() as Foo>::foo([(); 0]);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn associated_constant_refer_to_generic() {
    let src = r#"
    pub trait Deserialize {
        let N: u32;
        fn deserialize(fields: [Field; N]) -> Self;
    }

    impl<let M: u32> Deserialize for [Field; M] {
        let N: u32 = M;

        fn deserialize(fields: Self) -> Self {
            fields
        }
    }

    pub fn go<let M: u32>(fields: [Field; M]) {
        let _data = <[Field; M] as Deserialize<N = M>>::deserialize(fields);
    }
    "#;
    assert_no_errors(src);
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
fn associated_type_behind_self_as_trait_with_different_generics() {
    let src = r#"
    pub trait Foo<Baz> {
        type Bar;
        fn bar() -> <Self as Foo<u32>>::Bar;
                             ^^^ No matching impl found for `Self: Foo<u32, Bar = _>`
                             ~~~ No impl for `Self: Foo<u32, Bar = _>`
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

#[test]
fn associated_constant_in_trait_method_missing_in_impl() {
    // When an impl is missing an associated constant that is accessed in a default trait method,
    // we should only get ONE error (the "missing associated type" error)
    let src = "
    trait MyTrait {
        let N: u32;

        fn foo() {
            let _ = Self::N;
        }
    }

    impl MyTrait for i32 {
         ^^^^^^^ `MyTrait` is missing the associated type `N`
    }

    fn main() {}
    ";
    check_errors(src);
}

#[test]
fn associated_type_via_self_as_in_impl() {
    let src = r#"
    pub trait Foo {
        type Bar;
        fn foo() -> Self::Bar;
    }

    pub struct Qux;

    impl Foo for Qux {
        type Bar = u32;
        fn foo() -> <Self as Foo>::Bar { 10 }
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn generic_associated_type_access_direct_bound() {
    // T::Qux works when T: Baz and Baz defines Qux (direct bound syntax)
    let src = r#"
    trait Foo { type Bar; }
    trait Baz { type Qux; }

    impl<T: Baz> Foo for T {
        type Bar = T::Qux;
    }
    fn main() {}
    "#;
    assert_no_errors(src);
}

#[test]
fn associated_type_via_self_in_impl() {
    let src = r#"
    pub trait Foo {
        type Bar;
        fn foo() -> Self::Bar;
    }

    pub struct Qux;

    impl Foo for Qux {
        type Bar = u32;
        fn foo() -> Self::Bar { 10 }
    }"#;
    assert_no_errors(src);
}

#[test]
fn generic_associated_type_access_where_clause() {
    // T::Qux works when T: Baz and Baz defines Qux (where clause syntax)
    let src = r#"
    trait Foo { type Bar; }
    trait Baz { type Qux; }

    impl<T> Foo for T where T: Baz {
        type Bar = T::Qux;
    }
    fn main() {}
    "#;
    assert_no_errors(src);
}

#[test]
fn associated_type_referred_via_full_path_from_function() {
    let src = r#"
    pub trait Foo {
        type Bar;
    }

    pub struct Qux;

    impl Foo for Qux {
        type Bar = u32;
    }

    pub fn qux_foo_bar() -> <Qux as Foo>::Bar {
        0
    }
    fn main() {}
    "#;
    assert_no_errors(src);
}

#[test]
fn generic_associated_type_in_function_signature() {
    // T::Bar works in generic function signatures
    let src = r#"
    trait Foo { type Bar; }

    pub fn use_bar<T>(_x: T::Bar) where T: Foo {
    }
    fn main() {}
    "#;
    assert_no_errors(src);
}

#[test]
fn generic_associated_type_ambiguous() {
    // Error when multiple traits define the same associated type
    let src = r#"
    trait Foo { type Bar; }
    trait Trait1 { type Qux; }
    trait Trait2 { type Qux; }

    impl<T> Foo for T where T: Trait1 + Trait2 {
        type Bar = T::Qux;
                   ^^^^^^ Multiple applicable items in scope
                   ~~~~~~ Multiple traits which provide `Qux` are implemented and in scope: `Trait1`, `Trait2`
    }
    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn generic_associated_type_not_found() {
    // Error when no trait defines the associated type - falls through to normal resolution
    // which fails because 'T' is a generic
    let src = r#"
    trait Foo { type Bar; }
    trait Baz { type Other; }

    impl<T> Foo for T where T: Baz {
        type Bar = T::Qux;
                   ^ Could not resolve 'T' in path
    }
    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn associated_type_through_multiple_traits() {
    // T::Bar works when T: Foo and Foo has associated type Bar
    let src = "
    trait HasQux { type Qux; }
    trait Foo { type Bar: HasQux; }
    trait Result { type Output; }

    impl<T> Result for T where T: Foo {
        type Output = T::Bar;
    }
    fn main() {}
    ";
    assert_no_errors(src);
}

#[test]
fn associated_type_in_trait_impl_method_direct_bound() {
    // T::Bar in a trait impl method
    let src = "
    trait HasQux { type Qux; }
    trait Foo { type Bar: HasQux; }
    trait WithMethod {
        type Output;
        fn use_bar(x: Self::Output);
    }

    impl<T: Foo> WithMethod for T {
        type Output = T::Bar;
        fn use_bar(_x: T::Bar) {}
    }
    fn main() {}
    ";
    assert_no_errors(src);
}

#[test]
fn associated_type_in_trait_impl_method_where_clause() {
    // T::Bar in a trait impl method
    let src = "
    trait HasQux { type Qux; }
    trait Foo { type Bar: HasQux; }
    trait WithMethod {
        type Output;
        fn use_bar(x: Self::Output);
    }

    impl<T> WithMethod for T where T: Foo {
        type Output = T::Bar;
        fn use_bar(_x: T::Bar) {}
    }
    fn main() {}
    ";
    assert_no_errors(src);
}

#[test]
fn associated_type_accessed_through_self_in_trait_impl_method() {
    let src = "
    trait HasQux { type Qux; }
    trait Foo { type Bar: HasQux; }
    trait Result {
        type Output;
        fn use_bar(_x: Self::Output) {}
    }

    impl<T> Result for T where T: Foo {
        type Output = T::Bar;
        fn use_bar(_x: Self::Output) {}
    }
    fn main() { }
    ";
    check_errors(src);
}

/// TODO(https://github.com/noir-lang/noir/issues/11376): Switch to assert no errors once resolved
#[test]
fn fully_qualified_nested_associated_type() {
    let src = "
    trait HasQux { type Qux; }
    trait Foo { type Bar: HasQux; }
    trait Result { type Output; }

    impl<T> Result for T where T: Foo {
        type Output = <T::Bar as HasQux>::Qux;
                                 ^^^^^^ No matching impl found for `<T as Foo>::Bar: HasQux<Qux = _>`
                                 ~~~~~~ No impl for `<T as Foo>::Bar: HasQux<Qux = _>`
    }
    fn main() {}
    ";
    check_errors(src);
}

#[test]
fn associated_constant_references_generic_in_impl() {
    let src = r#"
    trait HasSize {
        let SIZE: u32;
    }

    struct Wrapper<T> {
        inner: T,
    }

    impl HasSize for Field {
        let SIZE: u32 = 1;
    }

    impl<T> HasSize for Wrapper<T> where T: HasSize {
        let SIZE: u32 = <T as HasSize>::SIZE;
    }

    fn main() {
        let _: u32 = <Wrapper<Field> as HasSize>::SIZE;
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn associated_constant_arithmetic_with_generic_param() {
    let src = r#"
    trait HasSize {
        let SIZE: u32;
    }

    struct Pair<A, B> {
        a: A,
        b: B,
    }

    impl HasSize for Field {
        let SIZE: u32 = 1;
    }

    impl HasSize for bool {
        let SIZE: u32 = 1;
    }

    impl<A, B> HasSize for Pair<A, B> where A: HasSize, B: HasSize {
        let SIZE: u32 = <A as HasSize>::SIZE + <B as HasSize>::SIZE;
    }

    fn main() {
        let _: u32 = <Pair<Field, bool> as HasSize>::SIZE;
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn associated_type_with_trait_bound_in_generic_fn() {
    let src = r#"
    trait ToField {
        fn to_field(self) -> Field;
    }

    trait Container {
        type Element: ToField;
        fn get(self) -> Self::Element;
    }

    impl ToField for u32 {
        fn to_field(self) -> Field {
            self as Field
        }
    }

    struct MyBox {
        value: u32,
    }

    impl Container for MyBox {
        type Element = u32;
        fn get(self) -> Self::Element {
            self.value
        }
    }

    fn extract_as_field<C>(c: C) -> Field where C: Container {
        c.get().to_field()
    }

    fn main() {
        let b = MyBox { value: 42 };
        assert(extract_as_field(b) == 42);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn associated_type_equals_generic_param_in_impl() {
    let src = r#"
    trait Mappable {
        type Item;
        fn first(self) -> Self::Item;
    }

    struct List<T> {
        head: T,
    }

    impl<T> Mappable for List<T> {
        type Item = T;
        fn first(self) -> Self::Item {
            self.head
        }
    }

    fn get_head<T>(list: List<T>) -> T {
        list.first()
    }

    fn main() {
        let l = List { head: 42 as Field };
        assert(get_head(l) == 42);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn associated_constant_as_array_length_in_generic_fn() {
    let src = r#"
    trait Sized {
        let SIZE: u32;
    }

    struct Packet {}

    impl Sized for Packet {
        let SIZE: u32 = 4;
    }

    fn make_buffer<T>() -> [Field; <T as Sized>::SIZE] where T: Sized {
        [0; <T as Sized>::SIZE]
    }

    fn main() {
        let buf: [Field; 4] = make_buffer::<Packet>();
        assert(buf[0] == 0);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn numeric_generic_constrained_by_associated_constant() {
    let src = r#"
    trait HasLen {
        let LEN: u32;
    }

    struct MyArray<let N: u32> {
        data: [Field; N],
    }

    impl<let N: u32> HasLen for MyArray<N> {
        let LEN: u32 = N;
    }

    fn check_len<T>() -> u32 where T: HasLen {
        <T as HasLen>::LEN
    }

    fn main() {
        assert(check_len::<MyArray<5>>() == 5);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn associated_constant_used_in_generic_function_body() {
    let src = r#"
    trait Config {
        let MAX: u32;
        let MIN: u32;
    }

    struct Settings {}

    impl Config for Settings {
        let MAX: u32 = 100;
        let MIN: u32 = 0;
    }

    fn range<T>() -> u32 where T: Config {
        <T as Config>::MAX - <T as Config>::MIN
    }

    fn main() {
        assert(range::<Settings>() == 100);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_with_comptime_and_associated_constant() {
    let src = r#"
    trait TypeInfo {
        let SIZE: u32;
        comptime fn type_id() -> u32;
    }

    struct MyStruct {}

    impl TypeInfo for MyStruct {
        let SIZE: u32 = 10;
        comptime fn type_id() -> u32 {
            42
        }
    }

    fn get_size<T>() -> u32 where T: TypeInfo {
        <T as TypeInfo>::SIZE
    }

    fn main() {
        assert(get_size::<MyStruct>() == 10);
        comptime {
            assert(MyStruct::type_id() == 42);
        }
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_with_numeric_generic_and_associated_type() {
    let src = r#"
    trait Serializable<let N: u32> {
        type Aux;
        fn serialize(self) -> [Field; N];
    }

    struct Point {
        x: Field,
        y: Field,
    }

    impl Serializable<2> for Point {
        type Aux = bool;
        fn serialize(self) -> [Field; 2] {
            [self.x, self.y]
        }
    }

    fn ser<T, let N: u32>(val: T) -> [Field; N] where T: Serializable<N> {
        val.serialize()
    }

    fn main() {
        let p = Point { x: 1, y: 2 };
        let arr = ser(p);
        assert(arr[0] == 1);
        assert(arr[1] == 2);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn numeric_generic_with_trait_bound_and_arithmetic() {
    let src = r#"
    trait Sized {
        let SIZE: u32;
    }

    impl Sized for Field {
        let SIZE: u32 = 1;
    }

    fn double_size<T>() -> u32 where T: Sized {
        <T as Sized>::SIZE * 2
    }

    fn main() {
        assert(double_size::<Field>() == 2);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn numeric_generic_in_associated_constant_with_arithmetic() {
    let src = r#"
    trait StorageInfo {
        let TOTAL_SIZE: u32;
    }

    struct DoubleArray<let N: u32> {
        data: [Field; N],
    }

    impl<let N: u32> StorageInfo for DoubleArray<N> {
        let TOTAL_SIZE: u32 = N * 2;
    }

    fn get_storage_size<T>() -> u32 where T: StorageInfo {
        <T as StorageInfo>::TOTAL_SIZE
    }

    fn main() {
        assert(get_storage_size::<DoubleArray<5>>() == 10);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn associated_type_in_generic_impl() {
    let src = r#"
    trait Mappable {
        type Item;
        fn first(self) -> Self::Item;
    }

    struct List<T> {
        head: T,
    }

    impl<T> Mappable for List<T> {
        type Item = T;
        fn first(self) -> Self::Item {
            self.head
        }
    }

    fn get_head<T>(list: List<T>) -> T {
        list.first()
    }

    fn main() {
        let l = List { head: 42 as Field };
        assert(get_head(l) == 42);
    }
    "#;
    assert_no_errors(src);
}

/// Regression test for https://github.com/noir-lang/noir/issues/11545
#[test]
fn associated_type_shorthand_in_return_type() {
    let src = r#"
    trait Transform {
        type Output;
        fn transform(self) -> Self::Output;
    }

    impl Transform for Field {
        type Output = Field;
        fn transform(self) -> Self::Output {
            self
        }
    }

    fn apply_transform<T>(w: T) -> T::Output where T: Transform {
        w.transform()
    }

    fn main() {
        let _: Field = apply_transform(1 as Field);
    }
    "#;
    assert_no_errors(src);
}

/// Regression test for https://github.com/noir-lang/noir/issues/11545
#[test]
fn associated_type_shorthand_in_return_type_with_trait_having_constant() {
    let src = r#"
    trait Collection {
        type Item;
        let MAX_SIZE: u32;

        fn get(self, index: u32) -> Self::Item;
    }

    struct FieldVec {
        data: [Field; 4],
    }

    impl Collection for FieldVec {
        type Item = Field;
        let MAX_SIZE: u32 = 4;

        fn get(self, index: u32) -> Self::Item {
            self.data[index]
        }
    }

    fn first_element<C>(c: C) -> C::Item where C: Collection {
        c.get(0)
    }

    fn main() {
        let v = FieldVec { data: [10, 20, 30, 40] };
        let _ = first_element(v);
    }
    "#;
    assert_no_errors(src);
}

/// Regression test for https://github.com/noir-lang/noir/issues/11545
#[test]
fn associated_type_shorthand_simple_identity() {
    let src = r#"
    trait HasItem {
        type Item;
        fn item(self) -> Self::Item;
    }

    impl HasItem for Field {
        type Item = bool;
        fn item(self) -> Self::Item {
            true
        }
    }

    fn get_item<T>(t: T) -> T::Item where T: HasItem {
        t.item()
    }

    fn main() {
        let _ = get_item(1 as Field);
    }
    "#;
    assert_no_errors(src);
}

/// TODO(https://github.com/noir-lang/noir/issues/11545): remove should_panic once fixed
#[test]
#[should_panic(expected = "Expected no errors")]
fn associated_type_of_generic_in_param_position() {
    // Bug: M::Key can't be used as parameter type
    let src = r#"
    trait KeyType {
        type Key;
    }

    trait Lookup: KeyType {
        fn lookup(self, key: Self::Key) -> Field;
    }

    struct Map {
        key: Field,
        value: Field,
    }

    impl KeyType for Map {
        type Key = Field;
    }

    impl Lookup for Map {
        fn lookup(self, key: Self::Key) -> Field {
            if self.key == key { self.value } else { 0 }
        }
    }

    fn find<M>(m: M, key: M::Key) -> Field where M: Lookup {
        m.lookup(key)
    }

    fn main() {
        let m = Map { key: 1, value: 42 };
        assert(find(m, 1) == 42);
    }
    "#;
    assert_no_errors(src);
}

/// TODO(https://github.com/noir-lang/noir/issues/11545): remove should_panic once fixed
#[test]
#[should_panic(expected = "Expected no errors")]
fn associated_type_shorthand_in_param_position() {
    let src = r#"
    trait Container {
        type Item;
        fn contains(self, item: Self::Item) -> bool;
    }

    struct Bag {
        val: Field,
    }

    impl Container for Bag {
        type Item = Field;
        fn contains(self, item: Self::Item) -> bool {
            self.val == item
        }
    }

    fn check<C>(c: C, item: C::Item) -> bool where C: Container {
        c.contains(item)
    }

    fn main() {
        let b = Bag { val: 42 };
        assert(check(b, 42));
    }
    "#;
    assert_no_errors(src);
}

/// Regression test for https://github.com/noir-lang/noir/issues/11549
#[test]
fn nested_associated_type_access_fails() {
    // Bug: nested associated type resolution fails
    let src = r#"
    trait HasInner {
        type Inner;
    }

    trait HasValue {
        type Value;
        fn get_value(self) -> Self::Value;
    }

    struct A {}
    struct B {}

    impl HasValue for B {
        type Value = Field;
        fn get_value(self) -> Self::Value { 0 }
    }

    impl HasInner for A {
        type Inner = B;
    }

    fn process<T>(val: <T as HasInner>::Inner) -> <<T as HasInner>::Inner as HasValue>::Value
    where
        T: HasInner,
        <T as HasInner>::Inner: HasValue,
    {
        val.get_value()
    }

    fn main() {
        let b = B {};
        let _: Field = process::<A>(b);
    }
    "#;
    assert_no_errors(src);
}

/// Regression test for https://github.com/noir-lang/noir/issues/11545
#[test]
fn associated_type_in_generic_function_local_var() {
    let src = r#"
    trait HasItem {
        type Item;
        fn get(self) -> Self::Item;
    }

    struct Holder {
        val: Field,
    }

    impl HasItem for Holder {
        type Item = Field;
        fn get(self) -> Self::Item {
            self.val
        }
    }

    fn extract<T>(t: T) -> Field where T: HasItem {
        let x: T::Item = t.get();
        let _ = x;
        0
    }

    fn main() {
        let h = Holder { val: 42 };
        let _ = extract(h);
    }
    "#;
    assert_no_errors(src);
}

/// TODO(https://github.com/noir-lang/noir/issues/11545): remove should_panic once fixed
#[test]
#[should_panic(expected = "Expected no errors")]
fn associated_type_shorthand_used_as_struct_field_type() {
    // Bug: T::Item as a field type in a generic struct fails
    let src = r#"
    trait HasItem {
        type Item;
    }

    impl HasItem for Field {
        type Item = bool;
    }

    struct Derived<T> where T: HasItem {
        val: T::Item,
    }

    fn main() {
        let d: Derived<Field> = Derived { val: true };
        assert(d.val);
    }
    "#;
    assert_no_errors(src);
}

/// Regression test for https://github.com/noir-lang/noir/issues/11545
#[test]
fn generic_impl_with_associated_type_in_method_signature() {
    let src = r#"
    trait HasItem {
        type Item;
        fn get_item(self) -> Self::Item;
    }

    impl HasItem for Field {
        type Item = bool;
        fn get_item(self) -> Self::Item {
            self != 0
        }
    }

    struct Processor<T> {
        source: T,
    }

    impl<T> Processor<T> where T: HasItem {
        fn process(self) -> T::Item {
            self.source.get_item()
        }
    }

    fn main() {
        let p = Processor { source: 42 as Field };
        assert(p.process());
    }
    "#;
    assert_no_errors(src);
}

/// Regression test for https://github.com/noir-lang/noir/issues/11550
#[test]
fn generic_fn_returning_tuple_with_associated_type() {
    let src = r#"
    trait HasOutput {
        type Out;
        fn produce(self) -> Self::Out;
    }

    impl HasOutput for Field {
        type Out = bool;
        fn produce(self) -> Self::Out {
            self != 0
        }
    }

    fn produce_pair<T>(a: T, b: T) -> (T::Out, T::Out) where T: HasOutput {
        (a.produce(), b.produce())
    }

    fn main() {
        let (x, y) = produce_pair(1 as Field, 0 as Field);
        assert(x);
        assert(!y);
    }
    "#;
    assert_no_errors(src);
}

/// TODO(https://github.com/noir-lang/noir/issues/11551): remove should_panic once fixed
#[test]
#[should_panic(expected = "Expected no errors")]
fn trait_with_associated_type_used_in_other_method_signature() {
    // Bug: Associated type from one trait method used in another's signature
    let src = r#"
    trait Mappable {
        type Target;
        fn map_to(self) -> Self::Target;
    }

    trait Chainable: Mappable {
        fn chain(self) -> <Self::Target as Mappable>::Target where Self::Target: Mappable;
    }

    impl Mappable for Field {
        type Target = bool;
        fn map_to(self) -> Self::Target {
            self != 0
        }
    }

    impl Mappable for bool {
        type Target = u32;
        fn map_to(self) -> Self::Target {
            if self { 1 } else { 0 }
        }
    }

    impl Chainable for Field {
        fn chain(self) -> <Self::Target as Mappable>::Target where Self::Target: Mappable {
            self.map_to().map_to()
        }
    }

    fn main() {
        let x: Field = 5;
        let result = x.chain();
        assert(result == 1);
    }
    "#;
    assert_no_errors(src);
}

/// Regression test for https://github.com/noir-lang/noir/issues/11538
#[test]
fn associated_constant_can_reference_generic_from_trait_bound() {
    let src = r#"
    pub trait E {
        let x: u32;
    }

    pub struct A<F> {
        pub f: F,
    }

    impl<X: E> E for A<X> {
        let x: u32 = X::x;
    }

    fn main() {}
    "#;
    assert_no_errors(src);
}
