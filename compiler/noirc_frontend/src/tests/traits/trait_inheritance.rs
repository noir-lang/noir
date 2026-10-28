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

/// Regression test for https://github.com/noir-lang/noir-claude/issues/2055
///
/// A supertrait bound that mentions `Self` explicitly (`trait Child: Parent<Self>`) means "every
/// implementor of `Child` also implements `Parent` with itself as the argument" - so resolving a
/// method through that parent bound must substitute `Self` for the actual implementing type.
/// Leaving the bound as `Parent<Child's own Self>` makes any impl of `Parent` for that type look
/// like it satisfies the bound, even one whose generic argument doesn't match (`Narrow` here
/// implements both `Parent<Narrow>` and `Parent<Wide>`; only the former should apply). The type
/// checker accepts the program either way - elaboration never picks a concrete impl for a method
/// called on a still-generic `X: Child` - so the bug only surfaces at monomorphization, once `X`
/// is substituted with a concrete type that has more than one `Parent` impl to choose from.
#[test]
fn supertrait_bound_mentioning_self_is_instantiated_for_the_implementing_type() {
    let src = r#"
    trait Parent<T> {
        fn limit(self) -> Field;
    }
    trait Child: Parent<Self> {}

    struct Narrow { v: Field }
    struct Wide { v: Field }

    impl Parent<Wide> for Wide {
        fn limit(self) -> Field { self.v }
    }
    impl Parent<Narrow> for Narrow {
        fn limit(self) -> Field { self.v }
    }
    impl Parent<Wide> for Narrow {
        fn limit(self) -> Field { self.v }
    }
    impl Child for Wide {}
    impl Child for Narrow {}

    fn run<X: Child>(x: X) -> Field { x.limit() }

    fn main(v: Field) -> pub Field { run(Wide { v: 0 }) + run(Narrow { v }) }
    "#;
    assert_no_errors(src);
    get_monomorphized(src).expect(
        "Child: Parent<Self> should narrow to Narrow: Parent<Narrow> specifically, not be ambiguous with the unrelated Narrow: Parent<Wide>",
    );
}

/// With `trait Child: Parent<Self>`, `Y: Child + Parent<Wide>` gives `Y` both `Parent<Y>` and
/// `Parent<Wide>`, so the argument of `pick` decides which of the two a call uses.
#[test]
fn method_call_argument_picks_between_written_and_implied_bound_on_the_same_trait() {
    let src = r#"
    trait Parent<T> {
        fn limit(self) -> Field;
        fn pick(self, o: T) -> Field;
    }
    trait Child: Parent<Self> {}

    struct Narrow { v: Field }
    struct Wide { v: Field }

    impl Parent<Wide> for Wide {
        fn limit(self) -> Field { self.v }
        fn pick(self, o: Wide) -> Field { let _ = self; o.v }
    }
    impl Parent<Narrow> for Narrow {
        fn limit(self) -> Field { assert(self.v != 100); self.v }
        fn pick(self, o: Narrow) -> Field { let _ = self; o.v + 1 }
    }
    impl Parent<Wide> for Narrow {
        fn limit(self) -> Field { self.v }
        fn pick(self, o: Wide) -> Field { let _ = self; o.v + 2 }
    }
    impl Child for Wide {}
    impl Child for Narrow {}

    fn pick_written<Y: Child + Parent<Wide>>(y: Y, w: Wide) -> Field { y.pick(w) }
    fn pick_implied<Y: Parent<Wide> + Child>(y: Y, other: Y) -> Field { y.pick(other) }
    fn run<X: Child>(x: X) -> Field { x.limit() }

    fn main(v: Field) -> pub Field {
        pick_written(Narrow { v }, Wide { v }) + pick_implied(Narrow { v }, Narrow { v })
            + run(Narrow { v })
    }
    "#;
    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    fn main$f0(v$l0: Field) -> pub Field {
        ((pick_written$f1({
            let v$l1 = v$l0;
            (v$l1)
        }, {
            let v$l2 = v$l0;
            (v$l2)
        }) + pick_implied$f2({
            let v$l3 = v$l0;
            (v$l3)
        }, {
            let v$l4 = v$l0;
            (v$l4)
        })) + run$f3({
            let v$l5 = v$l0;
            (v$l5)
        }))
    }
    fn pick_written$f1(y$l6: (Field,), w$l7: (Field,)) -> Field {
        pick$f4(y$l6, w$l7)
    }
    fn pick_implied$f2(y$l8: (Field,), other$l9: (Field,)) -> Field {
        pick$f5(y$l8, other$l9)
    }
    fn run$f3(x$l10: (Field,)) -> Field {
        limit$f6(x$l10)
    }
    fn pick$f4(self$l11: (Field,), o$l12: (Field,)) -> Field {
        let _$l13 = self$l11;
        (o$l12.0 + 2)
    }
    fn pick$f5(self$l14: (Field,), o$l15: (Field,)) -> Field {
        let _$l16 = self$l14;
        (o$l15.0 + 1)
    }
    fn limit$f6(self$l17: (Field,)) -> Field {
        assert((self$l17.0 != 100));;
        self$l17.0
    }
    ");
}

#[test]
fn method_call_that_does_not_pick_between_two_bounds_on_the_same_trait_is_ambiguous() {
    let src = r#"
    trait Parent<T> {
        fn limit(self) -> Field;
    }
    trait Child: Parent<Self> {}

    fn limit_of<Y: Child + Parent<u8>>(y: Y) -> Field { y.limit() }
       ^^^^^^^^ unused function limit_of
       ~~~~~~~~ unused function
                                                        ^^^^^^^ Multiple trait impls match the object type `Y`
                                                        ~~~~~~~ Ambiguous impl

    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn method_call_that_does_not_pick_between_two_written_bounds_on_the_same_trait_is_ambiguous() {
    let src = r#"
    trait Parent<T> {
        fn limit(self) -> Field;
    }

    fn limit_of<Y: Parent<u8> + Parent<u16>>(y: Y) -> Field { y.limit() }
       ^^^^^^^^ unused function limit_of
       ~~~~~~~~ unused function
                                                              ^^^^^^^ Multiple trait impls match the object type `Y`
                                                              ~~~~~~~ Ambiguous impl

    fn main() {}
    "#;
    check_errors(src);
}

/// Regression test for https://github.com/noir-lang/noir-claude/issues/1232
///
/// `X::get` under `X: Child<u16>` with `trait Child<U>: Parent<U>` must resolve through
/// `X: Parent<u16>`, so it returns `u16`. Inside a `Child` default body the parent bound must not
/// be left as `Parent<U>`, where `U` is the enclosing trait's own generic.
#[test]
fn trait_path_through_generic_supertrait_uses_the_bound_arguments() {
    let src = r#"
    trait Parent<T> {
        fn get(self) -> T;
    }
    trait Child<U>: Parent<U> {
        fn check<X: Child<u16>>(_self: Self, x: X) -> u16 {
            X::get(x)
        }
    }
    pub struct S {}
    impl Parent<u8> for S {
        fn get(self) -> u8 { 8 }
    }
    impl Parent<u16> for S {
        fn get(self) -> u16 { 16 }
    }
    impl Child<u8> for S {}
    impl Child<u16> for S {}

    fn main() -> pub u16 {
        <S as Child<u8>>::check(S {}, S {})
    }
    "#;
    assert_no_errors(src);
    get_monomorphized(src).expect("X::get under X: Child<u16> should resolve to S: Parent<u16>");
}

/// Regression test for https://github.com/noir-lang/noir-claude/issues/1232
///
/// In a free function nothing binds `Child`'s own generic, so an uninstantiated parent bound
/// `T: Parent<B>` reaches monomorphization unresolved.
#[test]
fn trait_path_through_generic_supertrait_in_free_function_monomorphizes() {
    let src = r#"
    trait Parent<A> {
        fn marker(self) -> Field;
    }
    trait Child<B>: Parent<B> {}
    struct Wrapper {}
    impl Parent<u32> for Wrapper {
        fn marker(self) -> Field { 10 }
    }
    impl Parent<bool> for Wrapper {
        fn marker(self) -> Field { 20 }
    }
    impl Child<u32> for Wrapper {}

    fn via_static_type<T>(x: T) -> Field where T: Child<u32> {
        T::marker(x)
    }

    fn main() -> pub Field {
        via_static_type(Wrapper {})
    }
    "#;
    assert_no_errors(src);
    get_monomorphized(src).expect("T::marker under T: Child<u32> should resolve to Parent<u32>");
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

/// Regression test for https://github.com/noir-lang/noir-claude/issues/1811
///
/// An associated constant elided from a supertrait bound keeps its numeric kind, so using it
/// where a type is expected is an error.
#[test]
fn elided_supertrait_associated_constant_keeps_its_kind() {
    let src = r#"
    trait Par {
        let N: u32;
    }
    trait Sub: Par {}

    pub fn f<T>(_x: <T as Par>::N) where T: Sub {}

    fn main() {}
    "#;
    let errors = get_program_errors(src);
    assert!(
        errors.iter().any(|error| format!("{error:?}").contains("TypeKindMismatch")),
        "expected a kind mismatch, got {errors:?}"
    );
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
fn elided_parent_associated_type_stays_rigid_after_explicit_parent() {
    let src = r#"
    pub trait Parent {
        type A;
    }
    pub trait Child1: Parent {}
    pub trait Child2: Parent {}
    pub struct Wide {}

    pub fn f<T>(x: <T as Parent>::A) -> Wide where T: Parent, T: Child1 {
                                        ^^^^ expected type Wide, found type <T as Parent>::A
                                        ~~~~ expected Wide because of return type
        x
        ~ <T as Parent>::A returned here
    }

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
fn default_method_uses_associated_type_its_parent_bound_leaves_out() {
    let src = r#"
    trait Level1 { type A; }
    trait Level2: Level1 { fn get_a(self) -> Self::A; fn twice(self) -> (Self::A, Self::A) { let a = self.get_a(); (a, a) } }
    pub struct D1 {}
    pub struct D2 {}
    impl Level1 for D1 { type A = Field; }
    impl Level1 for D2 { type A = bool; }
    impl Level2 for D1 { fn get_a(self) -> Field { 1 } }
    impl Level2 for D2 { fn get_a(self) -> bool { true } }
    fn main() {
        let (_x, _): (Field, Field) = D1 {}.twice();
        let (_y, _): (bool, bool) = D2 {}.twice();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn default_method_uses_associated_type_of_grandparent() {
    let src = r#"
    trait Level1 { type A; }
    trait Level2: Level1 { fn get_a(self) -> Self::A; }
    trait Level3: Level2 { fn twice(self) -> (Self::A, Self::A) { let a = self.get_a(); (a, a) } }
    pub struct D1 {}
    impl Level1 for D1 { type A = Field; }
    impl Level2 for D1 { fn get_a(self) -> Field { 1 } }
    impl Level3 for D1 {}
    fn main() {
        let (_x, _): (Field, Field) = D1 {}.twice();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn method_bound_on_unrelated_trait_uses_the_implementors_associated_type() {
    let src = r#"
    trait Conv { type O; fn conv(self) -> Self::O; }
    pub trait User {
        fn m(self) -> <Self as Conv>::O where Self: Conv { Conv::conv(self) }
        fn n(self) -> Field where Self: Conv { let _ = Conv::conv(self); 7 }
    }
    pub struct S { v: Field }
    impl Conv for S { type O = Field; fn conv(self) -> Field { self.v + 2 } }
    impl User for S {}
    fn main() {
        let _: Field = S { v: 1 }.m();
        let _: Field = S { v: 1 }.n();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn method_bound_on_unrelated_generic_trait_uses_the_implementors_associated_type() {
    let src = r#"
    trait Conv<X> { type O; fn conv(self, x: X) -> Self::O; }
    pub trait User {
        fn m(self) -> <Self as Conv<u16>>::O where Self: Conv<u16> { Conv::<u16>::conv(self, 2) }
    }
    pub struct S { v: Field }
    impl Conv<u16> for S { type O = Field; fn conv(self, x: u16) -> Field { self.v + x as Field } }
    impl User for S {}
    fn main() {
        let _: Field = S { v: 1 }.m();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn method_declaration_bound_on_unrelated_trait_is_callable_from_generic_code() {
    let src = r#"
    trait Conv { type O; fn conv(self) -> Self::O; }
    trait User { fn m(self) -> <Self as Conv>::O where Self: Conv; }
    pub struct S { v: Field }
    impl Conv for S { type O = Field; fn conv(self) -> Field { self.v + 2 } }
    impl User for S { fn m(self) -> Field { self.conv() * 10 } }
    fn generic<T>(t: T) -> <T as Conv>::O where T: User + Conv { t.m() }
    fn main() {
        let _: Field = S { v: 1 }.m();
        let _: Field = generic(S { v: 1 });
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn method_bounds_on_one_trait_with_different_arguments_have_distinct_associated_types() {
    let src = r#"
    trait Conv<X> { type O; fn conv(self, x: X) -> Self::O; }
    pub trait User {
        fn m(self) -> <Self as Conv<u8>>::O where Self: Conv<u8> + Conv<u16> {
            let a: <Self as Conv<u8>>::O = Conv::<u8>::conv(self, 1);
            let _b: <Self as Conv<u16>>::O = a;
                                             ^ Expected type Self::O, found type Self::O
            a
        }
    }
    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn method_bound_on_supertrait_with_other_arguments_has_its_own_associated_type() {
    let src = r#"
    trait Conv<X> { type O; fn conv(self, x: X) -> Self::O; }
    pub trait User: Conv<u8> {
        fn m(self) -> <Self as Conv<u16>>::O where Self: Conv<u16> { Conv::<u16>::conv(self, 2) }
    }
    pub struct S { v: Field }
    impl Conv<u8> for S { type O = bool; fn conv(self, _x: u8) -> bool { true } }
    impl Conv<u16> for S { type O = Field; fn conv(self, x: u16) -> Field { self.v + x as Field } }
    impl User for S {}
    fn main() {
        let _: Field = S { v: 1 }.m();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn method_bound_on_supertrait_with_other_arguments_is_a_distinct_projection() {
    let src = r#"
    trait Conv<X> { type O; fn conv(self, x: X) -> Self::O; }
    pub trait User: Conv<u8> {
        fn m(self) -> Field where Self: Conv<u16> {
            let a: <Self as Conv<u8>>::O = Conv::<u8>::conv(self, 1);
            let _b: <Self as Conv<u16>>::O = a;
                                             ^ Expected type Self::O, found type Self::O
            0
        }
    }
    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn method_bound_on_supertrait_uses_the_supertraits_associated_type() {
    let src = r#"
    trait Parent { type A; fn get(self) -> Self::A; }
    trait Marker {}
    trait Child: Parent {
        fn pair(self) -> (<Self as Parent>::A, <Self as Parent>::A) where Self: Parent + Marker {
            (self.get(), self.get())
        }
    }
    pub struct S { x: u8 }
    impl Parent for S { type A = u8; fn get(self) -> u8 { self.x + 1 } }
    impl Marker for S {}
    impl Child for S {}
    fn main() {
        let _: (u8, u8) = S { x: 1 }.pair();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_that_is_its_own_parent_is_a_cycle() {
    let src = r#"
    pub trait A: A {}
                 ^ Dependency cycle found
                 ~ 'A' recursively depends on itself: A -> A
    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn trait_that_is_its_own_parent_with_a_blanket_impl_is_a_cycle() {
    let src = r#"
    pub trait Bar {}
    pub trait A: A + Bar {}
                 ^ Dependency cycle found
                 ~ 'A' recursively depends on itself: A -> A
    impl<T> A for T where T: Bar {}
    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn trait_alias_of_itself_is_a_cycle() {
    let src = r#"
    pub trait Bar { fn bar(self) -> u32; }
    pub trait A = A + Bar;
                  ^ Dependency cycle found
                  ~ 'A' recursively depends on itself: A -> A
    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn trait_where_clause_on_its_own_self_is_a_cycle() {
    let src = r#"
    pub trait A where Self: A {}
                            ^ Dependency cycle found
                            ~ 'A' recursively depends on itself: A -> A
    fn main() {}
    "#;
    check_errors(src);
}

/// A parent bound is instantiated with one use's arguments for the trait's generics. Inside the
/// trait those arguments can themselves be the trait's generics (`U: Child<V, Field>` below), and
/// each one is used as written: `U`'s parent is `Parent<V>`, not `Parent<Field>`.
#[test]
fn parent_bound_arguments_naming_the_traits_own_generics_are_used_as_written() {
    let src = r#"
    trait Parent<T> { fn p(self) -> Field; }
    trait Child<U, V>: Parent<U> {
        fn c(self, u: U) -> Field where U: Child<V, Field> { let _ = self; u.p() }
    }
    struct A { v: Field }
    struct B { v: Field }
    impl Parent<B> for A { fn p(self) -> Field { assert(self.v != 100); self.v } }
    impl Parent<Field> for A { fn p(self) -> Field { self.v } }
    impl Child<B, Field> for A {}
    impl Parent<A> for B { fn p(self) -> Field { self.v } }
    impl Child<A, B> for B {}
    fn main(v: Field) -> pub Field { let b = B { v: 7 }; b.c(A { v }) }
    "#;
    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    fn main$f0(v$l0: Field) -> pub Field {
        let b$l2 = {
            let v$l1 = 7;
            (v$l1)
        };
        c$f1(b$l2, {
            let v$l3 = v$l0;
            (v$l3)
        })
    }
    fn c$f1(self$l4: (Field,), u$l5: (Field,)) -> Field {
        let _$l6 = self$l4;
        p$f2(u$l5)
    }
    fn p$f2(self$l7: (Field,)) -> Field {
        assert((self$l7.0 != 100));;
        self$l7.0
    }
    ");
}

#[test]
fn parent_bound_arguments_that_swap_the_traits_own_generics() {
    let src = r#"
    pub trait Parent<X> {}
    pub trait Child<A, B>: Parent<A> {
        fn swapped(self) where Self: Child<B, A> { let _ = self; }
    }
    fn main() {}
    "#;
    assert_no_errors(src);
}

#[test]
fn parent_bound_argument_that_is_the_generic_it_replaces() {
    let src = r#"
    trait Parent<T> { fn p(self) -> Field; }
    trait Child<U, V>: Parent<U> {
        fn c(self, u: U) -> Field where U: Child<V, U> { let _ = self; u.p() }
    }
    fn main() {}
    "#;
    assert_no_errors(src);
}

#[test]
fn associated_type_bound_arguments_that_swap_the_traits_own_generics() {
    let src = r#"
    pub trait Conv<X> {}
    pub trait Foo<A, B> {
        type Out: Conv<A>;
        fn m(self) where Self: Foo<B, A, Out = u8> { let _ = self; }
    }
    fn main() {}
    "#;
    assert_no_errors(src);
}

/// A trait reached through two parents at different arguments (`Quux<u8>`, `Quux<u16>`) is found
/// through either.
#[test]
fn second_parent_path_to_a_trait_at_other_arguments_is_found() {
    let src = r#"
    pub trait Quux<X> { fn quux_method() -> X; }
    pub trait Bar: Quux<u8> {}
    pub trait Baz: Quux<u16> {}
    pub trait Foo: Bar + Baz {}
    fn use_only_u16<T: Foo>() -> u16 { <T as Quux<u16>>::quux_method() }
    pub struct S {}
    impl Quux<u8> for S { fn quux_method() -> u8 { 7 } }
    impl Quux<u16> for S { fn quux_method() -> u16 { 13 } }
    impl Bar for S {}
    impl Baz for S {}
    impl Foo for S {}
    fn main() { let _ = use_only_u16::<S>(); }
    "#;
    assert_no_errors(src);
}

/// Inside `trait Child: Parent<Item = Field>`, `Self::Item` is `Field`.
#[test]
fn parent_bound_fixing_an_associated_type_is_used_in_a_default_method() {
    let src = r#"
    pub trait Parent { type Item; }
    pub trait Child: Parent<Item = Field> {
        fn to_field(self, item: Self::Item) -> Field { let _ = self; item }
    }
    pub struct S {}
    impl Parent for S { type Item = Field; }
    impl Child for S {}
    fn main() { let _ = S {}.to_field(1); }
    "#;
    assert_no_errors(src);
}

/// A parent bound fixing a grandparent's associated type (`Parent: Grand<Item = Field>`) fixes
/// `Self::Item` in a child trait.
#[test]
fn associated_type_fixed_by_a_grandparent_bound_is_used_in_a_default_method() {
    let src = r#"
    pub trait Grand { type Item; }
    pub trait Parent: Grand<Item = Field> {}
    pub trait Child: Parent {
        fn to_field(self, item: Self::Item) -> Field { let _ = self; item }
    }
    pub struct S {}
    impl Grand for S { type Item = Field; }
    impl Parent for S {}
    impl Child for S {}
    fn main() { let _ = S {}.to_field(1); }
    "#;
    assert_no_errors(src);
}

/// `Self::Item` is the type one parent fixes, whichever parent is listed first.
#[test]
fn associated_type_fixed_by_one_of_two_parents_is_used_in_either_order() {
    for parents in ["P1 + P2", "P2 + P1"] {
        let src = format!(
            r#"
    pub trait Grand {{ type Item; }}
    pub trait P1: Grand {{}}
    pub trait P2: Grand<Item = Field> {{}}
    pub trait Child: {parents} {{
        fn f(self, x: Self::Item) -> Field {{ let _ = self; x }}
    }}
    fn main() {{}}
    "#
        );
        assert_no_errors(&src);
    }
}

/// Two parents fixing an associated type to the same type agree on it.
#[test]
fn associated_type_fixed_to_one_type_by_two_parents_is_that_type() {
    let src = r#"
    pub trait Grand { type Item; }
    pub trait P1: Grand<Item = Field> {}
    pub trait P2: Grand<Item = Field> {}
    pub trait Child: P1 + P2 {
        fn f(self, x: Self::Item) -> Field { let _ = self; x }
    }
    fn main() {}
    "#;
    assert_no_errors(src);
}

/// No type implements a trait whose parents fix an associated type to different types, which is
/// an error on the trait, as in rustc (E0284).
#[test]
fn associated_type_fixed_to_different_types_by_two_parents_is_an_error() {
    let src = r#"
    pub trait Grand { type Item; }
    pub trait P1: Grand<Item = u8> {}
    pub trait P2: Grand<Item = Field> {}
    pub trait Child: P1 + P2 {}
              ^^^^^ The parent traits fix `<Self as Grand>::Item` to different types: `u8`, `Field`
              ~~~~~ No type can implement this trait
    fn main() {}
    "#;
    check_errors(src);
}

/// The same with one parent fixing it through its own associated type
/// (`P1: Grand<Item = Self::Out>`): only implementors with `Out = Field` could satisfy both, and
/// rustc rejects the trait too (E0284).
#[test]
fn associated_type_fixed_through_a_parents_own_and_to_a_type_is_an_error() {
    let src = r#"
    pub trait Grand { type Item; }
    pub trait P1: Grand<Item = Self::Out> { type Out; }
    pub trait P2: Grand<Item = Field> {}
    pub trait Child: P2 + P1 {}
              ^^^^^ The parent traits fix `<Self as Grand>::Item` to different types: `Field`, `Self::Out`
              ~~~~~ No type can implement this trait
    fn main() {}
    "#;
    check_errors(src);
}

/// With `trait P1: Grand<Item = Self::Out>`, `Self::Item` in `trait Child: P1` is `<Self as P1>::Out`,
/// which each implementor decides; here it is `u8`.
#[test]
fn item_fixed_through_a_parents_own_associated_type_is_that_type_in_a_default_method() {
    let src = r#"
    pub trait Plus { fn plus(self, other: Self) -> Self; }
    impl Plus for u8 { fn plus(self, other: u8) -> u8 { self + other } }
    pub trait Grand { type Item; }
    pub trait P1: Grand<Item = Self::Out> { type Out: Plus; }
    pub struct T {}
    impl Grand for T { type Item = u8; }
    impl P1 for T { type Out = u8; }
    pub trait Child: P1 { fn add(_self: Self, x: Self::Item, y: Self::Item) -> Self::Item { x.plus(y) } }
    impl Child for T {}
    fn main() -> pub u8 { T {}.add(200, 100) }
    "#;
    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    fn main$f0() -> pub u8 {
        add$f1({
            ()
        }, 200, 100)
    }
    fn add$f1(_self$l0: (), x$l1: u8, y$l2: u8) -> u8 {
        plus$f2(x$l1, y$l2)
    }
    fn plus$f2(self$l3: u8, other$l4: u8) -> u8 {
        (self$l3 + other$l4)
    }
    ");
}

/// `Self::Item`, fixed through `<Self as P1>::Out`, is a different type for each implementor.
#[test]
fn item_fixed_through_a_parents_own_associated_type_differs_per_implementor() {
    let src = r#"
    pub trait Grand { type Item; }
    pub trait P1: Grand<Item = Self::Out> { type Out; }
    pub struct T {}
    impl Grand for T { type Item = u8; }
    impl P1 for T { type Out = u8; }
    pub struct S {}
    impl Grand for S { type Item = Field; }
    impl P1 for S { type Out = Field; }
    pub trait Child: P1 { fn id(self, x: Self::Item) -> Self::Item { let _ = self; x } }
    impl Child for S {}
    impl Child for T {}
    fn main() -> pub Field { let _a: u8 = T {}.id(7); S {}.id(9) }
    "#;
    assert_no_errors(src);
}

/// Inside the trait, `Self::Item` fixed through `<Self as P1>::Out` is not any one concrete type.
#[test]
fn item_fixed_through_a_parents_own_associated_type_is_not_a_concrete_type() {
    let src = r#"
    pub trait Grand { type Item; }
    pub trait P1: Grand<Item = Self::Out> { type Out; }
    pub struct T {}
    impl Grand for T { type Item = u8; }
    impl P1 for T { type Out = u8; }
    pub trait Child: P1 { fn f(self) -> bool { let _ = self; let y: Self::Item = true; y } }
                                                                                 ^^^^ Expected type Self::Out, found type bool
                                        ^^^^ expected type bool, found type Self::Out
                                        ~~~~ expected bool because of return type
                                                                                       ~ Self::Out returned here
    impl Child for T {}
    fn main() -> pub bool { T {}.f() }
    
    "#;
    check_errors(src);
}

/// A call through an implementor takes its type for `Self::Item` (`u8` for `T`).
#[test]
fn item_fixed_through_a_parents_own_associated_type_takes_the_implementors_type() {
    let src = r#"
    pub trait Plus { fn plus(self, other: Self) -> Self; }
    impl Plus for u8 { fn plus(self, other: u8) -> u8 { self + other } }
    pub trait Grand { type Item; }
    pub trait P1: Grand<Item = Self::Out> { type Out: Plus; }
    pub struct T {}
    impl Grand for T { type Item = u8; }
    impl P1 for T { type Out = u8; }
    pub trait Child: P1 { fn add(_self: Self, x: Self::Item, y: Self::Item) -> Self::Item { x.plus(y) } }
    impl Child for T {}
    fn main(x: Field, y: Field) -> pub Field { T {}.add(x, y) }
                                                        ^ Expected type u8, found type Field
                                                           ^ Expected type u8, found type Field
                                       ^^^^^ expected type Field, found type u8
                                       ~~~~~ expected Field because of return type
                                               ~~~~~~~~~~~~~~ u8 returned here
    
    "#;
    check_errors(src);
}

/// In a generic caller (`T: Child`), `T::Item` is the type one parent fixes, whichever parent is
/// listed first, as it is inside the trait.
#[test]
fn associated_type_fixed_by_one_of_two_parents_is_that_type_for_a_generic_caller() {
    for parents in ["P1 + P2", "P2 + P1"] {
        let src = format!(
            r#"
    pub trait Grand {{ type Item; }}
    pub trait P1: Grand<Item = Field> {{}}
    pub trait P2: Grand {{}}
    pub trait Child: {parents} {{ fn id(self, x: Self::Item) -> Self::Item; }}
    fn g<T: Child>(t: T, x: T::Item) -> T::Item {{ t.id(x) }}
    pub struct S {{}}
    impl Grand for S {{ type Item = Field; }}
    impl P1 for S {{}}
    impl P2 for S {{}}
    impl Child for S {{ fn id(self, x: Field) -> Field {{ let _ = self; x }} }}
    fn main() {{ let _ = g(S {{}}, 3); }}
    "#
        );
        assert_no_errors(&src);
    }
}

/// In a generic caller (`T: Child`), an item a parent fixes through its own associated type
/// (`P1: Grand<Item = Self::Out>`) is that type: `T::Item` is `<T as P1>::Out`.
#[test]
fn item_fixed_through_a_parents_own_associated_type_is_that_type_for_a_generic_caller() {
    let src = r#"
    pub trait Grand { type Item; }
    pub trait P1: Grand<Item = Self::Out> { type Out; }
    pub trait Child: P1 { fn id(self, x: Self::Item) -> Self::Item; }
    fn g<T: Child>(t: T, x: T::Item) -> T::Item { t.id(x) }
    fn h<T: Child>(t: T, x: T::Out) -> T::Item { t.id(x) }
    pub struct S {}
    impl Grand for S { type Item = Field; }
    impl P1 for S { type Out = Field; }
    impl Child for S { fn id(self, x: Field) -> Field { let _ = self; x } }
    fn main() { let _ = g(S {}, 3); let _ = h(S {}, 3); }
    "#;
    assert_no_errors(src);
}

/// The same with the parent's own associated type nested in the item
/// (`P1: Grand<Item = [Self::Out; 2]>`): `T::Item` is `[<T as P1>::Out; 2]`.
#[test]
fn item_fixed_to_a_type_mentioning_a_parents_own_associated_type_for_a_generic_caller() {
    let src = r#"
    pub trait Grand { type Item; }
    pub trait P1: Grand<Item = [Self::Out; 2]> { type Out; }
    pub trait Child: P1 { fn id(self, x: Self::Item) -> Self::Item; }
    fn g<T: Child>(t: T, x: T::Item) -> T::Item { t.id(x) }
    fn h<T: Child>(t: T, x: [T::Out; 2]) -> T::Item { t.id(x) }
    pub struct S {}
    impl Grand for S { type Item = [Field; 2]; }
    impl P1 for S { type Out = Field; }
    impl Child for S { fn id(self, x: [Field; 2]) -> [Field; 2] { let _ = self; x } }
    fn main() { let _ = g(S {}, [1, 2]); let _ = h(S {}, [1, 2]); }
    "#;
    assert_no_errors(src);
}

/// A parent bound that fixes an associated type (`Child: P1<Item = Field>`) leaves the parent's
/// own parents in view: a grandparent declaring an item of the same name makes `Self::Item`
/// ambiguous, as in rustc (E0221).
#[test]
fn associated_type_a_parent_fixes_is_ambiguous_with_a_grandparents_of_the_same_name() {
    let src = r#"
    pub trait Grand { type Item; }
    pub trait P1: Grand { type Item; }
    pub trait Child: P1<Item = Field> {
        fn f(self, x: Self::Item) -> Field { let _ = self; x }
                      ^^^^^^^^^^ Multiple applicable items in scope
                      ~~~~~~~~~~ Multiple traits which provide `Item` are implemented and in scope: `Grand`, `P1`
    }
    fn main() {}
    "#;
    check_errors(src);
}

/// Each type an associated type is fixed to is listed once in the error, however many parents
/// fix it to that type.
#[test]
fn associated_type_fixed_to_one_type_by_two_of_three_parents_is_listed_once() {
    let src = r#"
    pub trait Grand { type Item; }
    pub trait P1: Grand<Item = u8> {}
    pub trait P2: Grand<Item = Field> {}
    pub trait P3: Grand<Item = Field> {}
    pub trait Child: P1 + P2 + P3 {}
              ^^^^^ The parent traits fix `<Self as Grand>::Item` to different types: `u8`, `Field`
              ~~~~~ No type can implement this trait
    fn main() {}
    "#;
    check_errors(src);
}

/// In a generic caller, an item one parent fixes through its own associated type
/// (`P1: Grand<Item = Self::Out>`) is that type even when another parent (`P3: Grand`) reaches
/// the same trait first.
#[test]
fn item_fixed_through_a_parents_own_associated_type_for_a_generic_caller_in_either_order() {
    for parents in ["P1 + P3", "P3 + P1"] {
        let src = format!(
            r#"
    pub trait Grand {{ type Item; }}
    pub trait P1: Grand<Item = Self::Out> {{ type Out; }}
    pub trait P3: Grand {{}}
    pub trait Child: {parents} {{}}
    fn g<T: Child>(x: T::Out) -> <T as Grand>::Item {{ x }}
    pub struct S {{}}
    impl Grand for S {{ type Item = Field; }}
    impl P1 for S {{ type Out = Field; }}
    impl P3 for S {{}}
    impl Child for S {{}}
    fn main() {{ let _ = g::<S>(3); }}
    "#
        );
        assert_no_errors(&src);
    }
}

/// Two paths to a parent with several associated types agree item by item, whatever order each
/// bound writes them in (`Grand<B = u8>` next to `Grand`).
#[test]
fn item_fixed_by_one_of_two_parents_is_matched_by_name_for_a_generic_caller() {
    for parents in ["P1 + P3", "P3 + P1"] {
        let src = format!(
            r#"
    pub trait Grand {{ type A; type B; }}
    pub trait P3: Grand {{}}
    pub trait P1: Grand<B = u8> {{}}
    pub trait Child: {parents} {{}}
    fn k<T: Child>() {{}}
    fn g<T: Child>(x: <T as Grand>::B) -> u8 {{ x }}
    pub struct S {{}}
    impl Grand for S {{ type A = Field; type B = u8; }}
    impl P3 for S {{}}
    impl P1 for S {{}}
    impl Child for S {{}}
    fn main() {{ k::<S>(); let _ = g::<S>(1); }}
    "#
        );
        assert_no_errors(&src);
    }
}

/// Two parents agree when one fixes an item through its own associated type and the other fixes
/// that one (`P1: Grand<Item = Self::Out>` with `P1<Out = Field>`, next to
/// `P2: Grand<Item = Field>`): every implementor has `Item = Out = Field`.
#[test]
fn associated_type_fixed_through_a_parents_own_that_another_bound_fixes_is_not_an_error() {
    for parents in ["P1<Out = Field> + P2", "P2 + P1<Out = Field>"] {
        let src = format!(
            r#"
    pub trait Grand {{ type Item; }}
    pub trait P1: Grand<Item = Self::Out> {{ type Out; }}
    pub trait P2: Grand<Item = Field> {{}}
    pub trait Child: {parents} {{
        fn f(self, x: Self::Item) -> Field {{ let _ = self; x }}
    }}
    fn main() {{}}
    "#
        );
        assert_no_errors(&src);
    }
}

/// A conflict between two parents is reported on the trait that has both, not again on the
/// traits that inherit it, as in rustc.
#[test]
fn associated_type_conflict_is_reported_only_where_the_parents_meet() {
    let src = r#"
    pub trait Grand { type Item; }
    pub trait P1: Grand<Item = u8> {}
    pub trait P2: Grand<Item = Field> {}
    pub trait Mid: P1 + P2 {}
              ^^^ The parent traits fix `<Self as Grand>::Item` to different types: `u8`, `Field`
              ~~~ No type can implement this trait
    pub trait Child: Mid {}
    pub trait GrandChild: Child {}
    fn main() {}
    "#;
    check_errors(src);
}

/// A parent bound fixing an item to a type that doesn't resolve is reported once, as that type.
#[test]
fn associated_type_fixed_to_an_unknown_type_is_not_also_a_conflict() {
    let src = r#"
    pub trait Grand { type Item; }
    pub trait P1: Grand<Item = Missing> {}
                               ^^^^^^^ Could not resolve 'Missing' in path
    pub trait P2: Grand<Item = Field> {}
    pub trait Child: P1 + P2 {}
    fn main() {}
    "#;
    check_errors(src);
}
