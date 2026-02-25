//! Tests for trait inheritance (supertraits).
//! Validates that supertrait bounds are correctly enforced and resolved, including with generics.

use crate::{
    test_utils::stdlib_src,
    tests::{assert_no_errors, check_errors, check_errors_with_stdlib},
};

#[test]
fn trait_inheritance() {
    let src = r#"
        pub trait Foo {
            fn foo(self) -> Field;
        }

        pub trait Bar {
            fn bar(self) -> Field;
        }

        pub trait Baz: Foo + Bar {
            fn baz(self) -> Field;
        }

        pub fn foo<T>(baz: T) -> (Field, Field, Field) where T: Baz {
            (baz.foo(), baz.bar(), baz.baz())
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_inheritance_with_generics() {
    let src = r#"
        trait Foo<T> {
            fn foo(self) -> T;
        }

        trait Bar<U>: Foo<U> {
            fn bar(self);
        }

        pub fn foo<T>(x: T) -> i32 where T: Bar<i32> {
            x.foo()
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_inheritance_with_generics_2() {
    let src = r#"
        pub trait Foo<T> {
            fn foo(self) -> T;
        }

        pub trait Bar<T, U>: Foo<T> {
            fn bar(self) -> (T, U);
        }

        pub fn foo<T>(x: T) -> i32 where T: Bar<i32, i32> {
            x.foo()
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_inheritance_with_generics_3() {
    let src = r#"
        trait Foo<A> {}

        trait Bar<B>: Foo<B> {}

        impl Foo<i32> for () {}

        impl Bar<i32> for () {}
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_inheritance_with_generics_4() {
    let src = r#"
        trait Foo { type A; }

        trait Bar<B>: Foo<A = B> {}

        impl Foo for () { type A = i32; }

        impl Bar<i32> for () {}
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_inheritance_dependency_cycle() {
    let src = r#"
        trait Foo: Bar {}
              ^^^ Dependency cycle found
              ~~~ 'Foo' recursively depends on itself: Foo -> Bar -> Foo
        trait Bar: Foo {}
    "#;
    check_errors(src);
}

#[test]
fn removes_assumed_parent_traits_after_function_ends() {
    let src = r#"
    trait Foo {}
    trait Bar: Foo {}

    pub fn foo<T>()
    where
        T: Bar,
    {}

    pub fn bar<T>()
    where
        T: Foo,
    {}
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_inheritance_missing_parent_implementation() {
    let src = r#"
        pub trait Foo {}

        pub trait Bar: Foo {}
                       ~~~ required by this bound in `Bar`

        pub struct Struct {}

        impl Bar for Struct {}
                     ^^^^^^ The trait bound `Struct: Foo` is not satisfied
                     ~~~~~~ The trait `Foo` is not implemented for `Struct`

        fn main() {
        }
    "#;
    check_errors(src);
}

#[test]
// Regression test for https://github.com/noir-lang/noir/issues/6314
// Baz inherits from a single trait: Foo
fn regression_6314_single_inheritance() {
    let src = r#"
        trait Foo {
            fn foo(self) -> Self;
        }

        trait Baz: Foo {}

        impl<T> Baz for T where T: Foo {}

        fn main() { }
    "#;
    assert_no_errors(src);
}

#[test]
// Regression test for https://github.com/noir-lang/noir/issues/6314
// Baz inherits from two traits: Foo and Bar
fn regression_6314_double_inheritance() {
    let src = r#"
        trait Foo {
            fn foo(self) -> Self;
        }

        trait Bar {
            fn bar(self) -> Self;
        }

        trait Baz: Foo + Bar {}

        impl<T> Baz for T where T: Foo + Bar {}

        fn baz<T>(x: T) -> T where T: Baz {
            x.foo().bar()
        }

        impl Foo for Field {
            fn foo(self) -> Self {
                self + 1
            }
        }

        impl Bar for Field {
            fn bar(self) -> Self {
                self + 2
            }
        }

        fn main() {
            assert(0.foo().bar() == baz(0));
        }"#;

    assert_no_errors(src);
}

#[test]
fn trait_impl_with_child_constraint() {
    let src = r#"
    trait Parent {}

    trait Child: Parent {
        fn child() {}
    }

    pub struct Struct<T> {}

    impl<T: Parent> Parent for Struct<T> {}
    impl<T: Child> Child for Struct<T> {}
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_inheritance_with_ambiguous_associated_type() {
    let src = r#"
    pub trait Foo {
        type Bar;
        fn foo() -> Self::Bar;
    }

    pub trait Qux: Foo {
        type Bar;
        // This is rejected by Rust as ambiguous, but is accepted by Noir.
        fn qux() -> Self::Bar;

        fn quy() -> <Self as Qux>::Bar;
        fn quz() -> <Self as Foo>::Bar;
    }
    "#;
    check_errors(src);
}

#[test]
fn trait_inheritance_assoc_via_self_as_in_impl() {
    let src = r#"
    pub trait Foo {
        type Bar;
    }

    pub trait Qux: Foo {
        fn quz() -> <Self as Foo>::Bar;
    }

    pub struct Spam;

    impl Foo for Spam {
        type Bar = u32;
    }

    impl Qux for Spam {
        fn quz() -> <Self as Foo>::Bar {
            10
        }
    }

    fn main() {}
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_inheritance_assoc_disambiguate_via_self_as_in_impl() {
    let src = r#"
    pub trait Foo {
        type Bar;
        fn foo() -> Self::Bar;
    }

    pub trait Qux: Foo {
        type Bar;
        fn quy() -> <Self as Qux>::Bar;
        fn quz() -> <Self as Foo>::Bar;
    }

    pub struct Spam;

    impl Foo for Spam {
        type Bar = u32;
        fn foo() -> Self::Bar { 10 }
    }

    impl Qux for Spam {
        type Bar = str<5>;

        fn quy() -> <Self as Qux>::Bar {
            "hello"
        }
        fn quz() -> <Self as Foo>::Bar {
            <Self as Foo>::foo()
        }
    }

    fn main() {}
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_inheritance_using_eq_in_default_method() {
    let src = "
    pub trait Foo: Eq {
        fn foo(self) -> bool {
            self == self
        }
    }
    ";
    check_errors_with_stdlib(src, [stdlib_src::EQ]);
}

#[test]
fn trait_inheritance_with_calling_method_on_self_in_default_method() {
    let src = r#"
    pub trait Empty: Eq {
        fn empty() -> Self;

        fn is_empty(self) -> bool {
            self.eq(Self::empty())
        }
    }
    "#;
    check_errors_with_stdlib(src, [stdlib_src::EQ]);
}

#[test]
fn trait_self_bound_with_calling_method_on_self_in_default_method() {
    let src = r#"
    pub trait Empty
    where Self: Eq {
        fn empty() -> Self;

        fn is_empty(self) -> bool {
            self.eq(Self::empty())
        }
    }
    "#;
    check_errors_with_stdlib(src, [stdlib_src::EQ]);
}

#[test]
fn trait_inheritance_with_generic_impl_and_base_call() {
    let src = r#"
    trait Base {
        fn base_method(self) -> Field;
    }

    trait Extended: Base {
        fn extended_method(self) -> Field;
    }

    struct Data<T> {
        value: T,
    }

    impl Base for Data<Field> {
        fn base_method(self) -> Field {
            self.value
        }
    }

    impl Extended for Data<Field> {
        fn extended_method(self) -> Field {
            self.base_method() + 1
        }
    }

    fn use_extended<T>(t: T) -> Field where T: Extended {
        t.extended_method()
    }

    fn main() {
        let d = Data { value: 10 as Field };
        assert(use_extended(d) == 11);
    }
    "#;
    assert_no_errors(src);
}

// Known bug: Self::A from grandparent trait not accessible in impl

/// TODO(https://github.com/noir-lang/noir/issues/11547): remove should_panic once fixed
#[test]
#[should_panic(expected = "Expected no errors")]
fn supertrait_associated_type_in_impl() {
    // Bug: Self::Key from supertrait KeyType not resolved in Lookup impl
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

    fn main() {
        let m = Map { key: 1, value: 42 };
        assert(m.lookup(1) == 42);
    }
    "#;
    assert_no_errors(src);
}

/// TODO(https://github.com/noir-lang/noir/issues/11548): remove should_panic once fixed
#[test]
#[should_panic(expected = "Expected no errors")]
fn trait_inheritance_chain_with_associated_types() {
    // Bug: Self::A from grandparent trait Level1 not accessible in Level3 impl.
    // Self::B from parent trait Level2 also not accessible.
    let src = r#"
    trait Level1 {
        type A;
    }

    trait Level2: Level1 {
        type B;
        fn get_a(self) -> Self::A;
    }

    trait Level3: Level2 {
        fn get_b(self) -> Self::B;
    }

    struct Data {
        a: Field,
        b: bool,
    }

    impl Level1 for Data {
        type A = Field;
    }

    impl Level2 for Data {
        type B = bool;
        fn get_a(self) -> Self::A { self.a }
    }

    impl Level3 for Data {
        fn get_b(self) -> Self::B { self.b }
    }

    fn process<T>(t: T) -> Field where T: Level3 {
        t.get_a()
    }

    fn main() {
        let d = Data { a: 42, b: true };
        assert(process(d) == 42);
    }
    "#;
    assert_no_errors(src);
}
