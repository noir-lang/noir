//! Tests for trait inheritance (supertraits).
//! Validates that supertrait bounds are correctly enforced and resolved, including with generics.

use crate::{
    test_utils::{get_monomorphized, stdlib_src},
    tests::{assert_no_errors, check_errors, check_errors_with_stdlib, get_program_errors},
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

// Regression test for add_trait_bound_to_scope() cyclic recursion
#[test]
fn add_trait_bound_to_scope_dependency_cycle() {
    let src = r#"
        trait A: B {}
        trait B: C {}
        trait C: B {
              ^ Dependency cycle found
              ~ 'C' recursively depends on itself: C -> B -> C
            fn ping() -> u32;
        }

        pub fn foo<T: A>(_x: T) {}

        fn main() {}
    "#;
    check_errors(src);
}

// Regression test for find_methods_or_constants_in_trait() cyclic recursion
#[test]
fn find_methods_or_constants_in_trait_dependency_cycle() {
    let src = r#"
        trait A: B {}
        trait B: C {}
        trait C: B {
              ^ Dependency cycle found
              ~ 'C' recursively depends on itself: C -> B -> C
            fn ping() -> u32;
        }

        pub fn foo<T: A>() -> u32 {
            T::ping()
        }

        fn main() {}
    "#;
    check_errors(src);
}

// Regression test for lookup_methods_in_trait() cyclic recursion
#[test]
fn lookup_methods_in_trait_dependency_cycle() {
    let src = r#"
        trait A: B {}
        trait B: C {}
        trait C: B {
              ^ Dependency cycle found
              ~ 'C' recursively depends on itself: C -> B -> C
            fn ping(self) -> u32;
        }

        pub fn foo<T: A>(x: T) -> u32 {
            x.ping()
        }

        fn main() {}
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
        fn qux() -> Self::Bar;
                    ^^^^^^^^^ Multiple applicable items in scope
                    ~~~~~~~~~ Multiple traits which provide `Bar` are implemented and in scope: `Foo`, `Qux`

        fn quy() -> <Self as Qux>::Bar;
                     ^^^^ Multiple applicable items in scope
                     ~~~~ Multiple traits which provide `Bar` are implemented and in scope: `Foo`, `Qux`
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
    // Because Qux inherit from Foo, and they both define the associated type Bar
    // `<Self as Qux>::Bar` does not disambiguate `Bar`
    let src = r#"
    pub trait Foo {
        type Bar;
        fn foo() -> Self::Bar;
    }

    pub trait Qux: Foo {
        type Bar;
        fn quy() -> <Self as Qux>::Bar;
                     ^^^^ Multiple applicable items in scope
                     ~~~~ Multiple traits which provide `Bar` are implemented and in scope: `Foo`, `Qux`
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
    check_errors(src);
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

/// Regression test for <https://github.com/noir-lang/noir/issues/11547>.
/// A subtrait may name an associated type declared on its supertrait via the `Self::Key`
/// shorthand, both in the trait's method signature and in the implementing method.
#[test]
fn supertrait_associated_type_in_impl() {
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
            let _ = self.key;
            let _ = self.value;
            key
        }
    }

    fn main() {
        let m = Map { key: 1, value: 42 };
        let _ = m.lookup(1);
    }
    "#;
    assert_no_errors(src);
}

/// A trait may access associated types defined on any of its ancestor traits
/// (parent, grandparent, ...), and a generic function bounded by such a trait
/// can call methods whose signatures reference those inherited associated types.
#[test]
fn trait_inheritance_chain_with_associated_types() {
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

    fn process<T>(t: T) -> <T as Level1>::A where T: Level3 {
        t.get_a()
    }

    fn main() {
        let d = Data { a: 42, b: true };
        assert(process(d) == 42);
    }
    "#;
    assert_no_errors(src);
}

/// A method whose return type references the trait's own associated type can be called through
/// a grandchild bound, even when none of the intervening traits add associated types of their
/// own. The inherited associated type must still resolve via the grandchild's bound.
#[test]
fn grandparent_trait_method_returning_own_associated_type() {
    let src = r#"
    trait Level1 {
        type A;
        fn get_a(self) -> Self::A;
    }

    trait Level2: Level1 {}
    trait Level3: Level2 {}

    struct Data {
        a: Field,
    }

    impl Level1 for Data {
        type A = Field;
        fn get_a(self) -> Self::A { self.a }
    }

    impl Level2 for Data {}
    impl Level3 for Data {}

    fn process<T>(t: T) -> <T as Level1>::A where T: Level3 {
        t.get_a()
    }

    fn main() {
        let d = Data { a: 42 };
        assert(process(d) == 42);
    }
    "#;
    assert_no_errors(src);
}

/// The inherited associated type resolves across an arbitrarily deep inheritance chain, not just
/// a single grandparent hop.
#[test]
fn trait_inheritance_chain_with_associated_types_four_levels() {
    let src = r#"
    trait Level1 { type A; }
    trait Level2: Level1 {
        type B;
        fn get_a(self) -> Self::A;
    }
    trait Level3: Level2 { type C; }
    trait Level4: Level3 {
        fn get_c(self) -> Self::C;
    }

    struct Data {
        a: Field,
        b: bool,
        c: u32,
    }

    impl Level1 for Data { type A = Field; }
    impl Level2 for Data {
        type B = bool;
        fn get_a(self) -> Self::A { self.a }
    }
    impl Level3 for Data { type C = u32; }
    impl Level4 for Data {
        fn get_c(self) -> Self::C { self.c }
    }

    fn process<T>(t: T) -> <T as Level1>::A where T: Level4 {
        t.get_a()
    }

    fn main() {
        let d = Data { a: 42, b: true, c: 7 };
        assert(process(d) == 42);
    }
    "#;
    assert_no_errors(src);
}

/// Diamond trait inheritance should not report "Multiple traits in scope"
/// when the same trait method is reachable through multiple parent paths.
///     C       (defines foo)
///    / \
///   A   B     (both inherit C)
///    \ /
///     D      (inherits A + B)
#[test]
fn diamond_trait_inheritance_method_call() {
    let src = r#"
    trait C {
        fn foo(self) -> Field;
    }

    trait A: C {}
    trait B: C {}
    trait D: A + B {}

    fn call_foo<T: D>(x: T) -> Field {
        x.foo()
    }

    struct S {}

    impl C for S {
        fn foo(self) -> Field { 42 }
    }
    impl A for S {}
    impl B for S {}
    impl D for S {}

    fn main() {
        assert(call_foo(S {}) == 42);
    }
    "#;
    let errors = get_program_errors(src);
    let actual_errors: Vec<_> = errors.iter().filter(|e| e.is_error()).collect();
    assert!(actual_errors.is_empty(), "Expected no errors, got: {actual_errors:?}");
}

// Regression test for lookup_associated_type_in_parent_impls() cyclic recursion.
// Self::X inside the impl of A triggers lookup_associated_type_in_parent_impls
// which traverses parent impls B -> C -> B -> ... and would hang without cycle detection.
#[test]
fn lookup_associated_type_in_parent_impls_dependency_cycle() {
    let src = r#"
        trait B: C {}
              ^ Dependency cycle found
              ~ 'B' recursively depends on itself: B -> C -> B
        trait C: B {}

        trait A: B {
            type Y;
        }

        impl C for Field {}

        impl B for Field {}

        impl A for Field {
            type Y = Self::X;
                     ^^^^ Could not resolve 'Self' in path
        }

        fn main() {}
    "#;
    check_errors(src);
}

/// Regression test for https://github.com/noir-lang/noir-claude/issues/1232
///
/// Same shape as above with an associated constant. Called from a `Child<u8>` instance, `X::LIMIT`
/// under `X: Child<u16>` must still read `Parent<u16>`'s value, not `Parent<u8>`'s.
#[test]
fn trait_constant_through_generic_supertrait_uses_the_bound_arguments() {
    let src = r#"
    trait Parent<T> {
        let LIMIT: u32;
    }
    trait Child<U>: Parent<U> {
        fn limit<X: Child<u16>>(_self: Self, _x: X) -> u32 {
            X::LIMIT
        }
    }
    pub struct S {}
    impl Parent<u8> for S {
        let LIMIT: u32 = 1000000;
    }
    impl Parent<u16> for S {
        let LIMIT: u32 = 100;
    }
    impl Child<u8> for S {}
    impl Child<u16> for S {}

    fn main() {
        comptime {
            assert_eq(<S as Child<u8>>::limit(S {}, S {}), 100);
        }
    }
    "#;
    assert_no_errors(src);
}

/// Regression test for https://github.com/noir-lang/noir-claude/issues/1811
///
/// `trait Sub: Par` leaves `Par::N` out, so `T: Sub` implies `T: Par<N = <T as Par>::N>` for
/// each `T` separately. Two implementors with different `N` must both be usable through `Sub`.
#[test]
fn elided_supertrait_associated_constant_is_per_use() {
    let src = r#"
    trait Par {
        let N: u32;
    }
    trait Sub: Par {}

    struct Three {}
    struct Five {}
    impl Par for Three {
        let N: u32 = 3;
    }
    impl Par for Five {
        let N: u32 = 5;
    }
    impl Sub for Three {}
    impl Sub for Five {}

    fn first<T>(xs: [Field; <T as Par>::N]) -> Field where T: Sub {
        xs[0]
    }

    fn main() -> pub Field {
        first::<Three>([1, 2, 3]) + first::<Five>([1, 2, 3, 4, 5])
    }
    "#;
    assert_no_errors(src);
    get_monomorphized(src).expect("each `T: Sub` should get its own `<T as Par>::N`");
}

/// Regression test for https://github.com/noir-lang/noir-claude/issues/1811
///
/// Under `T: Checked` with `trait Checked: Source`, `<T as Source>::Out` is an abstract type. A
/// function that equates it with a concrete type must be rejected, not bind the projection for
/// every other function in the program.
#[test]
fn elided_supertrait_associated_type_is_not_bound_by_a_function_body() {
    let src = r#"
    pub trait Source {
        type Out;
    }
    pub trait Checked: Source {}
    pub struct Narrow {}
    pub struct Wide {}

    pub fn unused_helper<T>(x: <T as Source>::Out) -> Wide where T: Checked {
                                                      ^^^^ expected type Wide, found type <T as Source>::Out
                                                      ~~~~ expected Wide because of return type
        x
        ~ <T as Source>::Out returned here
    }

    fn main() {}
    "#;
    check_errors(src);
}

/// Regression test for https://github.com/noir-lang/noir-claude/issues/1811
///
/// The associated type reached through a supertrait resolves to the implementor's own value, so
/// dispatch through it runs `Narrow`'s impl.
#[test]
fn elided_supertrait_associated_type_dispatches_to_the_implementor() {
    let src = r#"
    pub trait Policy {
        fn check() -> Field;
    }
    pub struct Narrow {}
    pub struct Wide {}
    impl Policy for Narrow {
        fn check() -> Field { 1 }
    }
    impl Policy for Wide {
        fn check() -> Field { 2 }
    }
    pub trait Source {
        type Out;
    }
    pub trait Checked: Source {}
    pub struct S {}
    impl Source for S {
        type Out = Narrow;
    }
    impl Checked for S {}

    pub fn checked<T>() -> Field where T: Checked, <T as Source>::Out: Policy {
        <<T as Source>::Out as Policy>::check()
    }

    fn main() {
        comptime {
            assert_eq(checked::<S>(), 1);
        }
    }
    "#;
    assert_no_errors(src);
}

/// Inside `impl Child<u16> for S`, `Self::Out` with `Out` declared on the parent of
/// `trait Child<U>: Parent<U>` is `<S as Parent<u16>>::Out`. `S` implements `Parent` twice, so the
/// parent bound must be instantiated with the impl's `u16` to pick the right one.
#[test]
fn self_associated_type_from_generic_parent_in_impl_uses_the_impl_arguments() {
    let src = r#"
    trait Parent<T> {
        type Out;
    }
    trait Child<U>: Parent<U> {
        fn get(self) -> Self::Out;
    }
    pub struct S {}
    impl Parent<u8> for S {
        type Out = u8;
    }
    impl Parent<u16> for S {
        type Out = u16;
    }
    impl Child<u16> for S {
        fn get(self) -> Self::Out {
            300
        }
    }

    fn main() -> pub u16 {
        <S as Child<u16>>::get(S {})
    }
    "#;
    assert_no_errors(src);
    get_monomorphized(src).expect("`Self::Out` in `impl Child<u16>` should be u16");
}

/// The trait method's declared `Self::Out` is checked against the impl's `u16` through the impl's
/// own parent bound `S: Parent<u16>`, not `S: Parent<U>`.
#[test]
fn trait_method_returning_generic_parent_associated_type_matches_impl() {
    let src = r#"
    trait Parent<T> {
        type Out;
    }
    trait Child<U>: Parent<U> {
        fn get(self) -> Self::Out;
    }
    pub struct S {}
    impl Parent<u8> for S {
        type Out = u8;
    }
    impl Parent<u16> for S {
        type Out = u16;
    }
    impl Child<u16> for S {
        fn get(self) -> u16 {
            300
        }
    }

    fn g<T>(t: T) -> <T as Parent<u16>>::Out where T: Child<u16> {
        t.get()
    }

    fn main() -> pub u16 {
        g(S {})
    }
    "#;
    assert_no_errors(src);
    get_monomorphized(src).expect("`T: Child<u16>` should return `<T as Parent<u16>>::Out`");
}

/// With `trait Child: Parent<Self>`, `Self::Out` in `impl Child for N` is `<N as Parent<N>>::Out`.
#[test]
fn self_associated_type_from_parent_mentioning_self_in_impl() {
    let src = r#"
    trait Parent<T> {
        type Out;
    }
    trait Child: Parent<Self> {
        fn get(self) -> Self::Out;
    }
    pub struct N {}
    impl Parent<N> for N {
        type Out = u8;
    }
    impl Child for N {
        fn get(self) -> Self::Out {
            3
        }
    }

    fn main() -> pub u8 {
        N {}.get()
    }
    "#;
    assert_no_errors(src);
    get_monomorphized(src).expect("`Self::Out` in `impl Child for N` should be u8");
}

/// Regression test for https://github.com/noir-lang/noir-claude/issues/1967
///
/// `Self::Out` inherited from the generic parent impl `impl<let M: u32> P for S<M>` is
/// `[u8; K]` inside `impl<let K: u32> C for S<K>`: the parent impl's `M` is substituted.
#[test]
fn self_associated_type_from_generic_parent_impl_is_instantiated() {
    let src = r#"
    trait P {
        type Out;
        fn p() -> u32;
    }
    trait C: P {
        fn g(a: Self::Out) -> u32;
    }
    struct S<let M: u32> {}
    fn mk<let N: u32>() -> [u8; N] {
        [0; N]
    }
    fn len<let N: u32>(_a: [u8; N]) -> u32 {
        N
    }

    impl<let M: u32> P for S<M> {
        type Out = [u8; M];
        fn p() -> u32 {
            0
        }
    }
    impl<let K: u32> C for S<K> {
        fn g(a: Self::Out) -> u32 {
            len(a) * 1000 + K
        }
    }

    fn main() -> pub u32 {
        let _ = mk::<1>();
        S::<8>::g([0; 8])
    }
    "#;
    assert_no_errors(src);
    get_monomorphized(src).expect("`S::<8>::g` should take `[u8; 8]`");
}

/// Regression test for https://github.com/noir-lang/noir-claude/issues/1967
///
/// From inside the parent impl, `S::<8>::g` expects `[u8; 8]`, so passing `[u8; M]` is an error,
/// and an inferred argument (`mk()`) is built with 8 elements even when called from `S<16>`.
#[test]
fn generic_parent_associated_type_is_not_the_callers_impl_generic() {
    let rejected = r#"
    trait P {
        type Out;
        fn p() -> u32;
    }
    trait C: P {
        fn g(a: Self::Out) -> u32;
    }
    struct S<let M: u32> {}
    fn mk<let N: u32>() -> [u8; N] {
        [0; N]
    }
    fn len<let N: u32>(_a: [u8; N]) -> u32 {
        N
    }

    impl<let M: u32> P for S<M> {
        type Out = [u8; M];
        fn p() -> u32 {
            let y: [u8; M] = mk();
            S::<8>::g(y)
                      ^ Expected type [u8; 8], found type [u8; M]
        }
    }
    impl<let K: u32> C for S<K> {
        fn g(a: Self::Out) -> u32 {
            len(a)
        }
    }

    fn main() -> pub u32 {
        S::<16>::p()
    }
    "#;
    check_errors(rejected);

    let inferred = r#"
    trait P {
        type Out;
        fn p() -> u32;
    }
    trait C: P {
        fn g(a: Self::Out) -> u32;
    }
    struct S<let M: u32> {}
    fn mk<let N: u32>() -> [u8; N] {
        [0; N]
    }
    fn len<let N: u32>(_a: [u8; N]) -> u32 {
        N
    }

    impl<let M: u32> P for S<M> {
        type Out = [u8; M];
        fn p() -> u32 {
            S::<8>::g(mk())
        }
    }
    impl<let K: u32> C for S<K> {
        fn g(a: Self::Out) -> u32 {
            len(a)
        }
    }

    fn main() {
        comptime {
            assert_eq(S::<16>::p(), 8);
        }
    }
    "#;
    assert_no_errors(inferred);
}

/// `T: C` with `trait C: A<u8> + B` and `trait B: A<u16>` implies both `T: A<u8>` and
/// `T: A<u16>`: two bounds on the same trait with different arguments.
#[test]
fn bound_implies_same_trait_with_different_arguments_through_two_parents() {
    let src = r#"
    trait A<T> {
        fn a(self) -> T;
    }
    trait B: A<u16> {}
    trait C: A<u8> + B {}
    fn f<T: C>(t: T) -> u16 {
        <T as A<u16>>::a(t)
    }
    fn g<T: C>(t: T) -> u8 {
        <T as A<u8>>::a(t)
    }
    struct S {}
    impl A<u8> for S {
        fn a(self) -> u8 {
            let _ = self;
            1
        }
    }
    impl A<u16> for S {
        fn a(self) -> u16 {
            let _ = self;
            2
        }
    }
    impl B for S {}
    impl C for S {}

    fn main() {
        let _ = f(S {});
        let _ = g(S {});
    }
    "#;
    assert_no_errors(src);
}

/// Under `impl<T> W<T> where T: Checked`, `<T as Source>::Out` is abstract.
#[test]
fn elided_supertrait_associated_type_is_rigid_in_inherent_impl() {
    let src = r#"
    pub trait Source {
        type Out;
    }
    pub trait Checked: Source {}
    pub struct Wide {}
    pub struct W<T> {}
    impl<T> W<T> where T: Checked {
        pub fn bad(x: <T as Source>::Out) -> Wide {
                                             ^^^^ expected type Wide, found type <T as Source>::Out
                                             ~~~~ expected Wide because of return type
            x
            ~ <T as Source>::Out returned here
        }
    }
    fn main() {}
    "#;
    check_errors(src);
}

/// Under `trait Foo<T> where T: Checked`, `<T as Source>::Out` is abstract in default methods.
#[test]
fn elided_supertrait_associated_type_is_rigid_in_trait_where_clause() {
    let src = r#"
    pub trait Source {
        type Out;
    }
    pub trait Checked: Source {}
    pub struct Wide {}
    pub trait Foo<T> where T: Checked {
        fn bad(x: <T as Source>::Out) -> Wide {
                                         ^^^^ expected type Wide, found type <T as Source>::Out
                                         ~~~~ expected Wide because of return type
            x
            ~ <T as Source>::Out returned here
        }
    }
    fn main() {}
    "#;
    check_errors(src);
}

/// In a default method of `trait Foo: Checked`, `Self::Out` (from `Source`) is abstract.
#[test]
fn elided_supertrait_associated_type_is_rigid_for_trait_self() {
    let src = r#"
    pub trait Source {
        type Out;
    }
    pub trait Checked: Source {}
    pub struct Wide {}
    pub trait Foo: Checked {
        fn bad(x: <Self as Source>::Out) -> Wide {
                                            ^^^^ expected type Wide, found type Self::Out
                                            ~~~~ expected Wide because of return type
            x
            ~ Self::Out returned here
        }
    }
    fn main() {}
    "#;
    check_errors(src);
}

/// For a trait method generic `X: Checked`, `<X as Source>::Out` is abstract.
#[test]
fn elided_supertrait_associated_type_is_rigid_for_trait_method_generic() {
    let src = r#"
    pub trait Source {
        type Out;
    }
    pub trait Checked: Source {}
    pub struct Wide {}
    pub trait Foo {
        fn bad<X>(x: <X as Source>::Out) -> Wide where X: Checked {
                                            ^^^^ expected type Wide, found type <X as Source>::Out
                                            ~~~~ expected Wide because of return type
            x
            ~ <X as Source>::Out returned here
        }
    }
    fn main() {}
    "#;
    check_errors(src);
}

/// The rigid `<T as Source>::Out` from `T: Checked` is the same type at every mention in one item,
/// and call sites instantiate it per call.
#[test]
fn elided_supertrait_associated_type_is_one_type_per_item() {
    let src = r#"
    pub trait Source {
        type Out;
    }
    pub trait Checked: Source {}
    pub struct Wide {}
    pub struct Narrow {}
    pub struct S {}
    impl Source for S {
        type Out = Narrow;
    }
    impl Checked for S {}
    pub struct W<T> {}
    impl<T> W<T> where T: Checked {
        pub fn same(x: <T as Source>::Out) -> <T as Source>::Out {
            x
        }
    }
    pub fn pair<T>(x: <T as Source>::Out, y: <T as Source>::Out) -> [<T as Source>::Out; 2]
    where
        T: Checked,
    {
        [x, y]
    }

    fn main() {
        let _ = W::<S>::same(Narrow {});
        let _ = pair::<S>(Narrow {}, Narrow {});
    }
    "#;
    assert_no_errors(src);
}

// Regression tests for https://github.com/noir-lang/noir-claude/issues/2055 and
// https://github.com/noir-lang/noir-claude/issues/1232
//
// A parent bound that mentions `Self` (`trait Child: Parent<Self>`) or the child trait's
// generics must be instantiated for the bounded type at every use: `X: Child` implies
// `X: Parent<X>`, and `X: Child<u16>` with `trait Child<U>: Parent<U>` implies `X: Parent<u16>`.

#[test]
fn parent_bound_mentioning_self_with_one_implementor() {
    let src = r#"
    trait MyEq<T> { fn eq2(self, o: T) -> bool; }
    trait MyOrd: MyEq<Self> { fn le(self, o: Self) -> bool; }
    impl MyEq<u8> for u8 { fn eq2(self, o: u8) -> bool { self == o } }
    impl MyOrd for u8 { fn le(self, o: u8) -> bool { self <= o } }
    fn main(v: u8, w: pub u8) -> pub bool { v.eq2(w) | v.le(w) }
    "#;
    assert_no_errors(src);
}

#[test]
fn parent_bound_mentioning_self_with_two_implementors() {
    let src = r#"
    trait MyEq<T> { fn eq2(self, o: T) -> bool; }
    trait MyOrd: MyEq<Self> { fn le(self, o: Self) -> bool; }
    impl MyEq<u8> for u8 { fn eq2(self, o: u8) -> bool { self == o } }
    impl MyOrd for u8 { fn le(self, o: u8) -> bool { self <= o } }
    impl MyEq<u16> for u16 { fn eq2(self, o: u16) -> bool { self == o } }
    impl MyOrd for u16 { fn le(self, o: u16) -> bool { self <= o } }
    fn main(v: u8) -> pub bool { v.eq2(v) | v.le(v) | (v as u16).le(3) }
    "#;
    assert_no_errors(src);
}

#[test]
fn parent_bound_mentioning_self_with_two_implementors_in_other_order() {
    let src = r#"
    trait MyEq<T> { fn eq2(self, o: T) -> bool; }
    trait MyOrd: MyEq<Self> { fn le(self, o: Self) -> bool; }
    impl MyEq<u16> for u16 { fn eq2(self, o: u16) -> bool { self == o } }
    impl MyOrd for u16 { fn le(self, o: u16) -> bool { self <= o } }
    impl MyEq<u8> for u8 { fn eq2(self, o: u8) -> bool { self == o } }
    impl MyOrd for u8 { fn le(self, o: u8) -> bool { self <= o } }
    fn main(v: u8) -> pub bool { v.eq2(v) | v.le(v) | (v as u16).le(3) }
    "#;
    assert_no_errors(src);
}

#[test]
fn parent_bound_mentioning_self_used_through_generic_method_call() {
    let src = r#"
    trait Parent<T> { fn check(self, o: T) -> Field; }
    trait Child: Parent<Self> {}
    struct N { v: Field }
    impl Parent<N> for N { fn check(self, _o: N) -> Field { self.v } }
    impl Child for N {}
    fn run<X: Child>(x: X, y: X) -> Field { x.check(y) }
    fn main(v: Field) -> pub Field { run(N { v }, N { v }) }
    "#;
    assert_no_errors(src);
}

#[test]
fn parent_bound_mentioning_self_used_through_generic_trait_path_call() {
    let src = r#"
    trait Parent<T> { fn check(self, o: T) -> Field; }
    trait Child: Parent<Self> {}
    struct N { v: Field }
    impl Parent<N> for N { fn check(self, _o: N) -> Field { self.v } }
    impl Child for N {}
    fn run<X: Child>(x: X, y: X) -> Field { X::check(x, y) }
    fn main(v: Field) -> pub Field { run(N { v }, N { v }) }
    "#;
    assert_no_errors(src);
}

#[test]
fn parent_bound_mentioning_self_with_child_method_on_generic() {
    let src = r#"
    trait Parent<T> { fn check(self, o: T) -> Field; }
    trait Child: Parent<Self> { fn c(self) -> Field { let _ = self; 1 } }
    struct N { v: Field }
    impl Parent<N> for N { fn check(self, _o: N) -> Field { self.v } }
    impl Child for N {}
    fn run<X: Child>(x: X) -> Field { x.c() }
    fn main(v: Field) -> pub Field { run(N { v }) }
    "#;
    assert_no_errors(src);
}

#[test]
fn parent_bound_mentioning_self_dispatches_to_impl_for_bounded_type() {
    // `run(Narrow { .. })` must call `impl Parent<Narrow> for Narrow`'s `limit`, the one with the
    // assertion, even though `pin` resolves a `Parent<Wide>` method through a `Child` bound.
    let src = r#"
    trait Parent<T> { fn limit(self) -> Field; fn pick(self, o: T) -> Field; }
    trait Child: Parent<Self> {}
    struct Narrow { v: Field }
    struct Wide { v: Field }
    impl Parent<Wide> for Wide {
        fn limit(self) -> Field { self.v }
        fn pick(self, o: Wide) -> Field { let _ = self; o.v }
    }
    impl Parent<Narrow> for Narrow {
        fn limit(self) -> Field { assert(self.v != 100); self.v }
        fn pick(self, o: Narrow) -> Field { let _ = self; o.v }
    }
    impl Parent<Wide> for Narrow {
        fn limit(self) -> Field { self.v }
        fn pick(self, o: Wide) -> Field { let _ = self; o.v }
    }
    impl Child for Wide {}
    impl Child for Narrow {}
    fn pin<Y: Parent<Wide> + Child>(y: Y, w: Wide) -> Field { y.pick(w) }
    fn run<X: Child>(x: X) -> Field { x.limit() }
    fn main(v: Field) -> pub Field {
        pin(Wide { v: 0 }, Wide { v: 0 }) + run(Wide { v: 0 }) + run(Narrow { v })
    }
    "#;
    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    fn main$f0(v$l0: Field) -> pub Field {
        ((pin$f1({
            let v$l1 = 0;
            (v$l1)
        }, {
            let v$l2 = 0;
            (v$l2)
        }) + run$f2({
            let v$l3 = 0;
            (v$l3)
        })) + run$f3({
            let v$l4 = v$l0;
            (v$l4)
        }))
    }
    fn pin$f1(y$l5: (Field,), w$l6: (Field,)) -> Field {
        pick$f4(y$l5, w$l6)
    }
    fn run$f2(x$l7: (Field,)) -> Field {
        limit$f5(x$l7)
    }
    fn run$f3(x$l8: (Field,)) -> Field {
        limit$f6(x$l8)
    }
    fn pick$f4(self$l9: (Field,), o$l10: (Field,)) -> Field {
        let _$l11 = self$l9;
        o$l10.0
    }
    fn limit$f5(self$l12: (Field,)) -> Field {
        self$l12.0
    }
    fn limit$f6(self$l13: (Field,)) -> Field {
        assert((self$l13.0 != 100));;
        self$l13.0
    }
    ");
}

/// Regression test for https://github.com/noir-lang/noir-claude/issues/1232
///
/// `X: Child<u16>` implies `X: Parent<u16>`, so `X::LIMIT` and `X::get` must come from
/// `impl Parent<u16> for S`, also inside `<S as Child<u8>>::check` where `Child`'s own generic is
/// `u8`.
#[test]
fn generic_supertrait_item_path_uses_arguments_of_bound() {
    let src = r#"
    trait Parent<T> {
        let LIMIT: u32;
        fn get(self) -> T;
    }
    trait Child<U>: Parent<U> {
        fn check<X: Child<u16>>(_self: Self, x: X) -> u16 {
            assert(X::LIMIT == 100);
            X::get(x)
        }
    }
    pub struct S { v: Field }
    impl Parent<u8> for S {
        let LIMIT: u32 = 1000000;
        fn get(self) -> u8 { let _ = self; 8 }
    }
    impl Parent<u16> for S {
        let LIMIT: u32 = 100;
        fn get(self) -> u16 { assert(self.v == 42); 16 }
    }
    impl Child<u8> for S {}
    impl Child<u16> for S {}
    fn main(v: Field) -> pub u16 {
        <S as Child<u8>>::check(S { v: 0 }, S { v })
    }
    "#;
    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    fn main$f0(v$l0: Field) -> pub u16 {
        check$f1({
            let v$l1 = 0;
            (v$l1)
        }, {
            let v$l2 = v$l0;
            (v$l2)
        })
    }
    fn check$f1(_self$l3: (Field,), x$l4: (Field,)) -> u16 {
        assert((100 == 100));;
        get$f2(x$l4)
    }
    fn get$f2(self$l5: (Field,)) -> u16 {
        assert((self$l5.0 == 42));;
        16
    }
    ");
}

/// Regression test for https://github.com/noir-lang/noir-claude/issues/1232
///
/// In a free function nothing binds `Child`'s own generic, so an uninstantiated parent bound
/// `T: Parent<B>` reaches monomorphization unresolved.
#[test]
fn generic_supertrait_item_path_in_free_function() {
    let src = r#"
    trait Parent<A> { fn marker(self) -> Field; }
    trait Child<B>: Parent<B> {}
    struct Wrapper {}
    impl Parent<u32> for Wrapper { fn marker(self) -> Field { let _ = self; 10 } }
    impl Parent<bool> for Wrapper { fn marker(self) -> Field { let _ = self; 20 } }
    impl Child<u32> for Wrapper {}
    fn via_static_type<T>(x: T) -> Field where T: Child<u32> { T::marker(x) }
    fn main() -> pub Field { via_static_type(Wrapper {}) }
    "#;
    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    fn main$f0() -> pub Field {
        via_static_type$f1({
            ()
        })
    }
    fn via_static_type$f1(x$l0: ()) -> Field {
        marker$f2(x$l0)
    }
    fn marker$f2(self$l1: ()) -> Field {
        let _$l2 = self$l1;
        10
    }
    ");
}

#[test]
fn parent_bound_mentioning_self_method_call_does_not_fix_self() {
    // `Y: Child` implies `Y: Parent<Y>`, so `y.pick` resolved through it takes a `Y`. Method
    // lookup uses the first bound on a trait, so this reports the same error as the explicit
    // `Y: Parent<Y> + Parent<Wide>`; it must not bind `Child`'s `Self` to `Wide`, which would
    // make `run` dispatch `Narrow` to `impl Parent<Wide> for Narrow`.
    let src = r#"
    trait Parent<T> { fn limit(self) -> Field; fn pick(self, o: T) -> Field; }
    trait Child: Parent<Self> {}
    struct Narrow { v: Field }
    struct Wide { v: Field }
    impl Parent<Wide> for Wide {
        fn limit(self) -> Field { self.v }
        fn pick(self, o: Wide) -> Field { let _ = self; o.v }
    }
    impl Parent<Narrow> for Narrow {
        fn limit(self) -> Field { assert(self.v != 100); self.v }
        fn pick(self, o: Narrow) -> Field { let _ = self; o.v }
    }
    impl Parent<Wide> for Narrow {
        fn limit(self) -> Field { self.v }
        fn pick(self, o: Wide) -> Field { let _ = self; o.v }
    }
    impl Child for Wide {}
    impl Child for Narrow {}
    fn pin<Y: Child + Parent<Wide>>(y: Y, w: Wide) -> Field { y.pick(w) }
                                                                     ^ Expected type Y, found type Wide
    fn run<X: Child>(x: X) -> Field { x.limit() }
    fn main(v: Field) -> pub Field { pin(Wide { v: 0 }, Wide { v: 0 }) + run(Narrow { v }) }
    "#;
    check_errors(src);
}

#[test]
fn parent_associated_type_in_impl_resolves_through_parent_bound_mentioning_self() {
    // In `impl Child for Narrow`, `Self::A` comes from `Narrow: Parent<Narrow>`, the parent bound
    // instantiated for this impl, not from any `Parent<_>` impl of `Narrow`.
    let src = r#"
    trait Parent<T> { type A; fn mk(self) -> Self::A; }
    trait Child: Parent<Self> { fn use_a(self, a: Self::A) -> Field; }
    struct Narrow { v: Field }
    struct Wide { v: Field }
    impl Parent<Narrow> for Narrow { type A = u8; fn mk(self) -> u8 { self.v as u8 } }
    impl Parent<Wide> for Narrow { type A = u64; fn mk(self) -> u64 { self.v as u64 } }
    impl Child for Narrow { fn use_a(self, a: Self::A) -> Field { let _ = self; a as Field } }
    fn main() -> pub Field { let _ = Wide { v: 0 }; Narrow { v: 3 }.use_a(7) }
    "#;
    assert_no_errors(src);
}

// Regression tests for https://github.com/noir-lang/noir-claude/issues/1811
//
// An associated item that a bound in a trait declaration leaves unspecified (the `N` of `Par` in
// `trait Sub: Par`) is unknown at every use of the bound, with its declared kind. Each use gets
// its own unknown: rigid inside a generic function, inferred for a concrete type. Using the bound
// once must not fix the item for any other use. The same holds for a bound on an associated type
// that mentions the trait's `Self` (`trait Foo { type Bar: Baz<Self>; }`).

#[test]
fn elided_parent_associated_constant_is_not_a_type() {
    let src = r#"
    trait Par { let N: u32; }
    trait Sub: Par {}
    pub fn f<T>(_x: <T as Par>::N) where T: Sub {}
                    ^^^^^^^^^^^^^ Expected type, found numeric generic
                    ~~~~~~~~~~~~~ not a type
    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn elided_parent_associated_type_is_not_a_length() {
    let src = r#"
    trait Par { type A; }
    trait Sub: Par {}
    pub fn f<T>(_x: [Field; <T as Par>::A]) where T: Sub {}
                    ^^^^^^^^^^^^^^^^^^^^^^ Type provided when a numeric generic was expected
                    ~~~~~~~~~~~~~~~~~~~~~~ the numeric generic is not of type `u32`
    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn elided_parent_associated_constant_is_rigid_in_each_function() {
    let src = r#"
    trait Par { let N: u32; }
    trait Sub: Par {}
    pub fn f<T>(xs: [Field; 3]) -> [Field; <T as Par>::N] where T: Sub { xs }
                                   ^^^^^^^^^^^^^^^^^^^^^^ expected type [Field; <T as Par>::N], found type [Field; 3]
                                   ~~~~~~~~~~~~~~~~~~~~~~ expected [Field; <T as Par>::N] because of return type
                                                                         ~~ [Field; 3] returned here
    pub fn g<T>(ys: [Field; 7]) -> [Field; <T as Par>::N] where T: Sub { ys }
                                   ^^^^^^^^^^^^^^^^^^^^^^ expected type [Field; <T as Par>::N], found type [Field; 7]
                                   ~~~~~~~~~~~~~~~~~~~~~~ expected [Field; <T as Par>::N] because of return type
                                                                         ~~ [Field; 7] returned here
    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn elided_parent_associated_constant_with_generated_impl() {
    let src = r#"
    trait Par { let N: u32; }
    trait Sub: Par {}
    #[gen_sub]
    struct S {}
    impl Par for S { let N: u32 = 3; }
    comptime fn gen_sub(_s: TypeDefinition) -> Quoted { quote { impl Sub for S {} } }
    pub fn poison<T, let M: u32>(xs: [Field; M]) -> [Field; M] where T: Sub {
                                                    ^^^^^^^^^^ expected type [Field; M], found type [Field; <T as Par>::N]
                                                    ~~~~~~~~~~ expected [Field; M] because of return type
        let ys: [Field; <T as Par>::N] = xs;
                                         ^^ Expected type [Field; <T as Par>::N], found type [Field; M]
        ys
        ~~ [Field; <T as Par>::N] returned here
    }
    fn main() -> pub Field {
        assert(<S as Par>::N == 3);
        let a: [Field; 5] = [1, 2, 3, 4, 5];
        let b = poison::<S, 5>(a);
        b[4]
    }
    "#;
    check_errors(src);
}

#[test]
fn elided_parent_associated_type_with_two_implementors() {
    let src = r#"
    trait KeyType { type Key; }
    trait Lookup: KeyType { fn lookup(self, key: Self::Key) -> Field; }
    struct Map { key: Field }
    struct Map2 { key: u32 }
    impl KeyType for Map { type Key = Field; }
    impl KeyType for Map2 { type Key = u32; }
    impl Lookup for Map { fn lookup(self, key: Self::Key) -> Field { let _ = self.key; key } }
    impl Lookup for Map2 { fn lookup(self, key: Self::Key) -> Field { let _ = self.key; key as Field } }
    fn main() -> pub Field { Map { key: 1 }.lookup(1) + Map2 { key: 2 }.lookup(3) }
    "#;
    assert_no_errors(src);
}

#[test]
fn grandparent_associated_type_is_rigid_without_a_bound_pinning_it() {
    let src = r#"
    trait Level1 { type A; }
    trait Level2: Level1 { fn get_a(self) -> Self::A; }
    trait Level3: Level2 {}
    pub fn process<T>(t: T) -> Field where T: Level3 { t.get_a() }
                               ^^^^^ expected type Field, found type <T as Level1>::A
                               ~~~~~ expected Field because of return type
                                                       ~~~~~~~~~ <T as Level1>::A returned here
    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn elided_parent_associated_type_dispatches_per_bounded_type() {
    let src = r#"
    trait Check { fn check(self) -> Field; }
    struct Narrow { v: Field }
    struct Wide { v: Field }
    impl Check for Narrow { fn check(self) -> Field { assert(self.v != 100); self.v } }
    impl Check for Wide { fn check(self) -> Field { self.v } }
    trait Par { type A; fn get(self) -> Self::A; }
    trait Sub: Par {}
    struct S { v: Field }
    struct T2 { v: Field }
    impl Par for S { type A = Narrow; fn get(self) -> Narrow { Narrow { v: self.v } } }
    impl Par for T2 { type A = Wide; fn get(self) -> Wide { Wide { v: self.v } } }
    impl Sub for S {}
    impl Sub for T2 {}
    fn run<X: Sub>(x: X) -> Field where <X as Par>::A: Check { x.get().check() }
    fn main(v: Field) -> pub Field { run(T2 { v: 0 }) + run(S { v }) }
    "#;
    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    fn main$f0(v$l0: Field) -> pub Field {
        (run$f1({
            let v$l1 = 0;
            (v$l1)
        }) + run$f2({
            let v$l2 = v$l0;
            (v$l2)
        }))
    }
    fn run$f1(x$l3: (Field,)) -> Field {
        check$f3(get$f4(x$l3))
    }
    fn run$f2(x$l4: (Field,)) -> Field {
        check$f5(get$f6(x$l4))
    }
    fn check$f3(self$l5: (Field,)) -> Field {
        self$l5.0
    }
    fn get$f4(self$l6: (Field,)) -> (Field,) {
        {
            let v$l7 = self$l6.0;
            (v$l7)
        }
    }
    fn check$f5(self$l8: (Field,)) -> Field {
        assert((self$l8.0 != 100));;
        self$l8.0
    }
    fn get$f6(self$l9: (Field,)) -> (Field,) {
        {
            let v$l10 = self$l9.0;
            (v$l10)
        }
    }
    ");
}

#[test]
fn associated_type_bound_mentioning_self_does_not_fix_self() {
    let src = r#"
    trait Baz<T> { fn pick(self, o: T) -> Field; }
    pub struct Wide { v: Field }
    struct Item { v: Field }
    impl Baz<Wide> for Item { fn pick(self, o: Wide) -> Field { let _ = o.v; self.v } }
    trait Foo { type Bar: Baz<Self>; fn bar(self) -> Self::Bar; }
    impl Foo for Wide { type Bar = Item; fn bar(self) -> Item { Item { v: self.v } } }
    pub fn pin<X: Foo>(x: X, w: Wide) -> Field { x.bar().pick(w) }
                                                              ^ Expected type X, found type Wide
    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn associated_type_bound_mentioning_self_dispatches_per_bounded_type() {
    let src = r#"
    trait Baz<T> { fn lim(self) -> Field; }
    struct Narrow { v: Field }
    struct Wide { v: Field }
    struct Item { v: Field }
    impl Baz<Narrow> for Item { fn lim(self) -> Field { assert(self.v != 100); self.v } }
    impl Baz<Wide> for Item { fn lim(self) -> Field { self.v } }
    trait Foo { type Bar: Baz<Self>; fn bar(self) -> Self::Bar; }
    impl Foo for Narrow { type Bar = Item; fn bar(self) -> Item { Item { v: self.v } } }
    impl Foo for Wide { type Bar = Item; fn bar(self) -> Item { Item { v: self.v } } }
    fn run<X: Foo>(x: X) -> Field { x.bar().lim() }
    fn main(v: Field) -> pub Field { run(Wide { v: 0 }) + run(Narrow { v }) }
    "#;
    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    fn main$f0(v$l0: Field) -> pub Field {
        (run$f1({
            let v$l1 = 0;
            (v$l1)
        }) + run$f2({
            let v$l2 = v$l0;
            (v$l2)
        }))
    }
    fn run$f1(x$l3: (Field,)) -> Field {
        lim$f3(bar$f4(x$l3))
    }
    fn run$f2(x$l4: (Field,)) -> Field {
        lim$f5(bar$f6(x$l4))
    }
    fn lim$f3(self$l5: (Field,)) -> Field {
        self$l5.0
    }
    fn bar$f4(self$l6: (Field,)) -> (Field,) {
        {
            let v$l7 = self$l6.0;
            (v$l7)
        }
    }
    fn lim$f5(self$l8: (Field,)) -> Field {
        assert((self$l8.0 != 100));;
        self$l8.0
    }
    fn bar$f6(self$l9: (Field,)) -> (Field,) {
        {
            let v$l10 = self$l9.0;
            (v$l10)
        }
    }
    ");
}

/// With `trait Child: Parent<Self>`, a function bounded by `Y: Parent<Wide> + Child` has two
/// `Parent` bounds on `Y`. Method lookup resolves through the first one written, so
/// `y.pick(w: Wide)` resolves through `Y: Parent<Wide>`.
#[test]
fn method_lookup_uses_the_first_written_bound_on_a_trait() {
    let src = r#"
    trait Parent<T> {
        fn pick(self, o: T) -> Field;
    }
    trait Child: Parent<Self> {}
    struct Wide {
        v: Field,
    }
    impl Parent<Wide> for Wide {
        fn pick(self, o: Wide) -> Field {
            let _ = self;
            o.v
        }
    }
    impl Child for Wide {}

    fn pin<Y: Parent<Wide> + Child>(y: Y, w: Wide) -> Field {
        y.pick(w)
    }

    fn main() -> pub Field {
        pin(Wide { v: 0 }, Wide { v: 1 })
    }
    "#;
    assert_no_errors(src);
}
