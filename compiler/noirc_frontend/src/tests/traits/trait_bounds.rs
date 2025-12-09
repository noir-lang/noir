//! Tests for trait bound checking and where clause validation.
//! Validates that trait bounds are satisfied and constraints on associated types are correctly checked.

use crate::tests::{assert_no_errors, check_errors};

#[test]
fn trait_impl_for_a_type_that_implements_another_trait() {
    let src = r#"
    trait One {
        fn one(self) -> i32;
    }

    impl One for i32 {
        fn one(self) -> i32 {
            self
        }
    }

    trait Two {
        fn two(self) -> i32;
    }

    impl<T> Two for T where T: One {
        fn two(self) -> i32 {
            self.one() + 1
        }
    }

    pub fn use_it<T>(t: T) -> i32 where T: Two {
        Two::two(t)
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_impl_for_a_type_that_implements_another_trait_with_another_impl_used() {
    let src = r#"
    trait One {
        fn one(self) -> i32;
    }

    impl One for i32 {
        fn one(self) -> i32 {
            let _ = self;
            1
        }
    }

    trait Two {
        fn two(self) -> i32;
    }

    impl<T> Two for T where T: One {
        fn two(self) -> i32 {
            self.one() + 1
        }
    }

    impl Two for u32 {
        fn two(self) -> i32 {
            let _ = self;
            0
        }
    }

    pub fn use_it(t: u32) -> i32 {
        Two::two(t)
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn check_trait_implemented_for_all_t() {
    let src = "
    trait Default2 {
        fn default2() -> Self;
    }

    trait Eq2 {
        fn eq2(self, other: Self) -> bool;
    }

    trait IsDefault {
        fn is_default(self) -> bool;
    }

    impl<T> IsDefault for T where T: Default2 + Eq2 {
        fn is_default(self) -> bool {
            self.eq2(T::default2())
        }
    }

    struct Foo {
        a: u64,
    }

    impl Eq2 for Foo {
        fn eq2(self, other: Foo) -> bool { self.a == other.a }
    }

    impl Default2 for u64 {
        fn default2() -> Self {
            0
        }
    }

    impl Default2 for Foo {
        fn default2() -> Self {
            Foo { a: Default2::default2() }
        }
    }

    fn main(a: Foo) -> pub bool {
        a.is_default()
    }";
    assert_no_errors(src);
}

// TODO: WIP (testing disabled Type::TraitAsType)
// #[test]
// fn check_trait_as_type_as_fn_parameter() {
//     let src = "
//     trait Eq2 {
//         fn eq2(self, other: Self) -> bool;
//     }
//
//     struct Foo {
//         a: u64,
//     }
//
//     impl Eq2 for Foo {
//         fn eq2(self, other: Foo) -> bool { self.a == other.a }
//     }
//
//     // `impl T` syntax is expected to be desugared to a `where` clause
//     fn test_eq(x: impl Eq2) -> bool {
//         x.eq2(x)
//     }
//
//     fn main(a: Foo) -> pub bool {
//         test_eq(a)
//     }";
//     assert_no_errors(src);
// }
//
// #[test]
// fn check_trait_as_type_as_two_fn_parameters() {
//     let src = "
//     trait Eq2 {
//         fn eq2(self, other: Self) -> bool;
//     }
//
//     trait Test {
//         fn test(self) -> bool;
//     }
//
//     struct Foo {
//         a: u64,
//     }
//
//     impl Eq2 for Foo {
//         fn eq2(self, other: Foo) -> bool { self.a == other.a }
//     }
//
//     impl Test for u64 {
//         fn test(self) -> bool { self == self }
//     }
//
//     // `impl T` syntax is expected to be desugared to a `where` clause
//     fn test_eq(x: impl Eq2, y: impl Test) -> bool {
//         x.eq2(x) == y.test()
//     }
//
//     fn main(a: Foo, b: u64) -> pub bool {
//         test_eq(a, b)
//     }";
//     assert_no_errors(src);
// }

#[test]
fn does_not_error_if_impl_trait_constraint_is_satisfied_for_concrete_type() {
    let src = r#"
        pub trait Greeter {
            fn greet(self);
        }

        pub trait Foo<T>
        where
            T: Greeter,
        {
            fn greet<U>(object: U)
            where
                U: Greeter,
            {
                object.greet();
            }
        }

        pub struct SomeGreeter;
        impl Greeter for SomeGreeter {
            fn greet(self) {}
        }

        pub struct Bar;

        impl Foo<SomeGreeter> for Bar {}
    "#;
    assert_no_errors(src);
}

#[test]
fn does_not_error_if_impl_trait_constraint_is_satisfied_for_type_variable() {
    let src = r#"
        pub trait Greeter {
            fn greet(self);
        }

        pub trait Foo<T> where T: Greeter {
            fn greet(object: T) {
                object.greet();
            }
        }

        pub struct Bar;

        impl<T> Foo<T> for Bar where T: Greeter {
        }

        fn main() {
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn errors_if_impl_trait_constraint_is_not_satisfied() {
    let src = r#"
        pub trait Greeter {
            fn greet(self);
        }

        pub trait Foo<T>
        where
            T: Greeter,
               ~~~~~~~ required by this bound in `Foo`
        {
            fn greet<U>(object: U)
            where
                U: Greeter,
            {
                object.greet();
            }
        }

        pub struct SomeGreeter;

        pub struct Bar;

        impl Foo<SomeGreeter> for Bar {}
                                  ^^^ The trait bound `SomeGreeter: Greeter` is not satisfied
                                  ~~~ The trait `Greeter` is not implemented for `SomeGreeter`
    "#;
    check_errors(src);
}

#[test]
fn errors_on_unknown_type_in_trait_where_clause() {
    let src = r#"
        pub trait Foo<T> where T: Unknown {}
                                  ^^^^^^^ Could not resolve 'Unknown' in path

        fn main() {
        }
    "#;
    check_errors(src);
}

#[test]
fn trait_bounds_which_are_dependent_on_generic_types_are_resolved_correctly() {
    // Regression test for https://github.com/noir-lang/noir/issues/6420
    let src = r#"
        trait Foo {
            fn foo(self) -> Field;
        }

        trait Bar<T>: Foo {
            fn bar(self) -> Field {
                self.foo()
            }
        }

        struct MyStruct<T> {
            inner: Field,
        }

        trait MarkerTrait {}
        impl MarkerTrait for Field {}

        // `MyStruct<T>` implements `Foo` only when its generic type `T` implements `MarkerTrait`.
        impl<T> Foo for MyStruct<T>
        where
            T: MarkerTrait,
        {
            fn foo(self) -> Field {
                let _ = self;
                42
            }
        }

        // We expect this to succeed as `MyStruct<T>` satisfies `Bar`'s trait bounds
        // of implementing `Foo` when `T` implements `MarkerTrait`.
        impl<T> Bar<T> for MyStruct<T>
        where
            T: MarkerTrait,
        {
            fn bar(self) -> Field {
                31415
            }
        }

        fn main() {
            let foo: MyStruct<Field> = MyStruct { inner: 42 };
            let _ = foo.bar();
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_bound_with_associated_constant() {
    let src = r#"
    pub trait Other {
        let N: u32;
    }

    pub trait Trait<T>
    where
        T: Other,
    {}

    impl Other for Field {
        let N: u32 = 1;
    }

    impl Trait<Field> for i32 {}
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_method_call_when_it_has_bounds_on_generic() {
    let src = r#"
    trait BigNum {}

    trait BigCurve<B>
    where
        B: BigNum,
    {
        fn new() -> Self;
    }

    pub fn foo<B: BigNum, Curve: BigCurve<B>>() {
        let _: Curve = BigCurve::new();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_bound_constraining_two_generics() {
    let src = r#"
    pub trait Foo<U> {}

    pub trait Baz<T, U>
    where
        T: Foo<U>,
    {}

    pub struct HasFoo1 {}
    impl Foo<()> for HasFoo1 {}

    pub struct HasBaz1 {}
    impl Baz<HasFoo1, ()> for HasBaz1 {}
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_where_clause_associated_type_constraint_expected_order() {
    let src = r#"
    pub trait BarTrait {}

    pub trait Foo {
        type Bar;
    }

    pub trait Baz<T>
    where
        T: Foo,
        <T as Foo>::Bar: BarTrait,
    {}

    pub struct HasBarTrait1 {}
    impl BarTrait for HasBarTrait1 {}

    pub struct HasFoo1 {}
    impl Foo for HasFoo1 {
        type Bar = HasBarTrait1;
    }

    pub struct HasBaz1 {}
    impl Baz<HasFoo1> for HasBaz1 {}
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_where_clause_associated_type_constraint_unexpected_order() {
    let src = r#"
    pub trait BarTrait {}

    pub trait Foo {
        type Bar;
    }

    pub trait Baz<T>
    where
        <T as Foo>::Bar: BarTrait,
        T: Foo,
    {}

    pub struct HasBarTrait1 {}
    impl BarTrait for HasBarTrait1 {}

    pub struct HasFoo1 {}
    impl Foo for HasFoo1 {
        type Bar = HasBarTrait1;
    }

    pub struct HasBaz1 {}
    impl Baz<HasFoo1> for HasBaz1 {}
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_bound_on_implementing_type() {
    let src = r#"
    struct GenericStruct<T> {
        inner: T,
    }

    trait Foo {
        fn foo() {}
    }

    impl Foo for Field {}

    impl<T: Foo> Foo for GenericStruct<T> {}

    trait Bar {
        fn bar();
    }

    impl<T> Bar for GenericStruct<T>
    where
        GenericStruct<T>: Foo,
    {
        fn bar() {
            <Self as Foo>::foo()
        }
    }
    
    fn main() {
        GenericStruct::<Field>::bar();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_constraint_on_tuple_type() {
    let src = r#"
    trait Foo<A> {
        fn foo(self, x: A) -> bool;
    }

    pub fn bar<T, U, V>(x: (T, U), y: V) -> bool where (T, U): Foo<V> {
        x.foo(y)
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_constraint_on_tuple_type_pub_crate() {
    let src = r#"
    pub(crate) trait Foo<A> {
        fn foo(self, x: A) -> bool;
    }

    pub fn bar<T, U, V>(x: (T, U), y: V) -> bool where (T, U): Foo<V> {
        x.foo(y)
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn short_syntax_for_trait_constraint_on_trait_generic() {
    let src = r#"
    pub trait Other {
        fn other(self) {
            let _ = self;
        }
    }

    pub trait Trait<T: Other> {
        fn foo(x: T) {
            x.other();
        }
    }

    fn main() {}
    "#;
    assert_no_errors(src);
}

#[test]
fn does_not_error_if_type_parameter_is_used_in_trait_bound_named_generic() {
    let src = r#"
    pub trait SomeTrait {}
    pub trait AnotherTrait {
        type AssocType;
    }

    impl<T, U> SomeTrait for T where T: AnotherTrait<AssocType=U> {}
    "#;
    assert_no_errors(src);
}
