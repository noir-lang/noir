#![cfg(test)]
use crate::{
    elaborator::UnstableFeature,
    monomorphization::errors::MonomorphizationError,
    test_utils::{
        GetProgramOptions, get_monomorphized, get_monomorphized_with_options,
        get_monomorphized_with_stdlib,
    },
    tests::check_monomorphization_error_using_features,
};

mod stdlib_src {
    pub(super) const ZEROED: &str = "
        #[builtin(zeroed)]
        pub fn zeroed<T>() -> T {}
    ";

    pub(super) const EQ: &str = "
        pub trait Eq {
            fn eq(self, other: Self) -> bool;
        }
    ";

    pub(super) const NEG: &str = "
        pub trait Neg {
            fn neg(self) -> Self;
        }
    ";

    pub(super) const ARRAY_LEN: &str = "
        impl<T, let N: u32> [T; N] {
            #[builtin(array_len)]
            pub fn len(self) -> u32 {}
        }
    ";

    pub(super) const CHECKED_TRANSMUTE: &str = "
        #[builtin(checked_transmute)]
        pub fn checked_transmute<T, U>(value: T) -> U {}
    ";

    // Note that in the stdlib these are all comptime functions, which I thought meant
    // that the comptime interpreter was used to evaluate them, however they do seem to
    // hit the `try_evaluate_call::try_evaluate_call`.
    // To make sure they are handled here, I removed the `comptime` for these tests.
    pub(super) const MODULUS: &str = "
        #[builtin(modulus_num_bits)]
        pub fn modulus_num_bits() -> u64 {}

        #[builtin(modulus_be_bits)]
        pub fn modulus_be_bits() -> [u1] {}

        #[builtin(modulus_le_bits)]
        pub fn modulus_le_bits() -> [u1] {}

        #[builtin(modulus_be_bytes)]
        pub fn modulus_be_bytes() -> [u8] {}

        #[builtin(modulus_le_bytes)]
        pub fn modulus_le_bytes() -> [u8] {}
    ";
}

#[test]
fn bounded_recursive_type_errors() {
    // We want to eventually allow bounded recursive types like this, but for now they are
    // disallowed because they cause a panic in convert_type during monomorphization.
    let src = "
        fn main() {
            let _tree: Tree<Tree<Tree<()>>> = Tree::Branch(
                                              ^^^^^^^^^^^^ Type `Tree<Tree<()>>` is recursive
                                              ~~~~~~~~~~~~ All types in Noir must have a known size at compile-time
                Tree::Branch(Tree::Leaf, Tree::Leaf),
                Tree::Branch(Tree::Leaf, Tree::Leaf),
            );
        }

        enum Tree<T> {
            Branch(T, T),
            Leaf,
        }
        ";
    let features = vec![UnstableFeature::Enums];
    check_monomorphization_error_using_features(src, &features, false);
}

#[test]
fn recursive_type_with_alias_errors() {
    // We want to eventually allow bounded recursive types like this, but for now they are
    // disallowed because they cause a panic in convert_type during monomorphization.
    //
    // In the future we could lower this type to:
    // struct OptOptUnit {
    //     is_some: Field,
    //     some: OptUnit,
    //     none: (),
    // }
    //
    // struct OptUnit {
    //     is_some: Field,
    //     some: (),
    //     none: (),
    // }
    let src = "
        fn main() {
            let _tree: Opt<OptAlias<()>> = Opt::Some(OptAlias::None);
                                           ^^^^^^^^^ Type `Opt<()>` is recursive
                                           ~~~~~~~~~ All types in Noir must have a known size at compile-time
        }

        type OptAlias<T> = Opt<T>;

        enum Opt<T> {
            Some(T),
            None,
        }
        ";
    let features = vec![UnstableFeature::Enums];
    check_monomorphization_error_using_features(src, &features, false);
}

#[test]
fn mutually_recursive_types_error() {
    // cSpell:disable
    let src = "
        fn main() {
            let _zero = Even::Zero;
        }

        enum Even {
            Zero,
            ^^^^ Type `Odd` is recursive
            ~~~~ All types in Noir must have a known size at compile-time
            Succ(Odd),
        }

        enum Odd {
            One,
            Succ(Even),
        }
        ";
    // cSpell:enable
    let features = vec![UnstableFeature::Enums];
    check_monomorphization_error_using_features(src, &features, false);
}

#[test]
fn mutually_recursive_types_with_structs_error() {
    // cSpell:disable
    let src = "
        fn main() {
            let _zero = Even::Zero;
        }

        enum Even {
            Zero,
            ^^^^ Type `EvenSucc` is recursive
            ~~~~ All types in Noir must have a known size at compile-time
            Succ(EvenSucc),
        }

        pub struct EvenSucc { inner: Odd }

        enum Odd {
            One,
            Succ(OddSucc),
        }

        pub struct OddSucc { inner: Even }
        ";

    // cSpell:enable
    let features = vec![UnstableFeature::Enums];
    check_monomorphization_error_using_features(src, &features, false);
}

#[test]
fn simple_closure_with_no_captured_variables() {
    let src = r#"
    fn main(y: call_data(0) Field) -> pub Field {
        let x = 1;
        let closure = || x;
        closure()
    }
    "#;

    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    fn main$f0(y$l0: call_data(0) Field) -> pub Field {
        let x$l1 = 1;
        let closure$l6 = ({
            let closure_variable$l3 = {
                let env$l2 = (x$l1);
                (env$l2, lambda$f1)
            };
            closure_variable$l3
        }, {
            let closure_variable$l5 = {
                let env$l4 = (x$l1);
                (env$l4, lambda$f2)
            };
            closure_variable$l5
        });
        {
            let tmp$l7 = closure$l6.0;
            tmp$l7.1(tmp$l7.0)
        }
    }
    fn lambda$f1(mut env$l2: (Field,)) -> Field {
        env$l2.0
    }
    unconstrained fn lambda$f2(mut env$l4: (Field,)) -> Field {
        env$l4.0
    }
    ");
}

/// Stress test for type propagation through very deep call chains. Type should propagate correctly to level 5.
#[test]
fn deep_call_chain() {
    let src = r#"
    pub fn main() -> pub u32 {
        level1(42)
    }

    fn level1<T1>(x: T1) -> T1 {
        level2(x)
    }

    fn level2<T2>(x: T2) -> T2 {
        level3(x)
    }

    fn level3<T3>(x: T3) -> T3 {
        level4(x)
    }

    fn level4<T4>(x: T4) -> T4 {
        level5(x)
    }

    fn level5<T5>(x: T5) -> T5 {
        x
    }
    "#;

    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    fn main$f0() -> pub u32 {
        level1$f1(42)
    }
    fn level1$f1(x$l0: u32) -> u32 {
        level2$f2(x$l0)
    }
    fn level2$f2(x$l1: u32) -> u32 {
        level3$f3(x$l1)
    }
    fn level3$f3(x$l2: u32) -> u32 {
        level4$f4(x$l2)
    }
    fn level4$f4(x$l3: u32) -> u32 {
        level5$f5(x$l3)
    }
    fn level5$f5(x$l4: u32) -> u32 {
        x$l4
    }
    ");
}

#[test]
fn generic_struct_through_deep_call_chain() {
    let src = r#"
    pub struct Wrapper<T> {
        value: T
    }

    pub fn main(x: u8) -> pub Wrapper<u8> {
        level1(Wrapper { value: x })
    }

    fn level1<T>(w: Wrapper<T>) -> Wrapper<T> {
        level2(w)
    }

    fn level2<U>(w: Wrapper<U>) -> Wrapper<U> {
        level3(w)
    }

    fn level3<V>(w: Wrapper<V>) -> Wrapper<V> {
        w
    }
    "#;

    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    fn main$f0(x$l0: u8) -> pub (u8,) {
        level1$f1({
            let value$l1 = x$l0;
            (value$l1)
        })
    }
    fn level1$f1(w$l2: (u8,)) -> (u8,) {
        level2$f2(w$l2)
    }
    fn level2$f2(w$l3: (u8,)) -> (u8,) {
        level3$f3(w$l3)
    }
    fn level3$f3(w$l4: (u8,)) -> (u8,) {
        w$l4
    }
    ");
}

#[test]
fn nested_generic_structs() {
    let src = r#"
    pub struct Outer<T> {
        inner: Inner<T>
    }

    pub struct Inner<U> {
        value: U
    }

    pub fn main() -> pub Outer<u32> {
        level1(Outer { inner: Inner { value: 10 } })
    }

    fn level1<A>(o: Outer<A>) -> Outer<A> {
        level2(o)
    }

    fn level2<B>(o: Outer<B>) -> Outer<B> {
        level3(o)
    }

    fn level3<C>(o: Outer<C>) -> Outer<C> {
        o
    }
    "#;

    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    fn main$f0() -> pub ((u32,),) {
        level1$f1({
            let inner$l0 = {
                let value$l1 = 10;
                (value$l1)
            };
            (inner$l0)
        })
    }
    fn level1$f1(o$l2: ((u32,),)) -> ((u32,),) {
        level2$f2(o$l2)
    }
    fn level2$f2(o$l3: ((u32,),)) -> ((u32,),) {
        level3$f3(o$l3)
    }
    fn level3$f3(o$l4: ((u32,),)) -> ((u32,),) {
        o$l4
    }
    ");
}

#[test]
fn mixed_constrained_unconstrained() {
    let src = r#"
    pub fn main(x: u8) -> pub u8 {
        level1(x)
    }

    fn level1<T>(x: T) -> T {
        // Safety: For testing type propagation in monomorphization
        unsafe {
            level2(x)
        }
    }

    unconstrained fn level2<U>(x: U) -> U {
        level3(x)
    }

    fn level3<V>(x: V) -> V {
        x
    }
    "#;

    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    fn main$f0(x$l0: u8) -> pub u8 {
        level1$f1(x$l0)
    }
    fn level1$f1(x$l1: u8) -> u8 {
        {
            level2$f2(x$l1)
        }
    }
    unconstrained fn level2$f2(x$l2: u8) -> u8 {
        level3$f3(x$l2)
    }
    unconstrained fn level3$f3(x$l3: u8) -> u8 {
        x$l3
    }
    ");
}

#[test]
fn static_trait_method_call_with_multiple_generics() {
    // Test that static trait method calls correctly bind trait generics to impl generics.
    // The trait has two generic parameters (T, U) that must be bound to (u8, u32).
    let src = r#"
    trait MyTrait<T, U> {
        fn foo(input: T) -> U;
    }

    struct Foo<A, B>;

    impl MyTrait<u8, u32> for Foo<u8, u32> {
        fn foo(input: u8) -> u32 {
            input as u32
        }
    }

    pub fn main(x: u8) -> pub u32 {
        Foo::<u8, u32>::foo(x)
    }
    "#;

    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    fn main$f0(x$l0: u8) -> pub u32 {
        foo$f1(x$l0)
    }
    fn foo$f1(input$l1: u8) -> u32 {
        (input$l1 as u32)
    }
    ");
}

#[test]
fn generic_struct_implementing_generic_trait() {
    // Make sure trait generics Input=X=u8 and Output=Y=u32 bind correctly.
    let src = r#"
    trait MyTrait<Input, Output> {
        fn foo(input: Input) -> Output;
    }

    struct Foo<A, B>;

    impl MyTrait<u8, u32> for Foo<u8, u32> {
        fn foo(input: u8) -> u32 {
            input as u32
        }
    }

    pub fn main(x: u8) -> pub u32 {
        Foo::<u8, u32>::foo(x)
    }
    "#;

    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    fn main$f0(x$l0: u8) -> pub u32 {
        foo$f1(x$l0)
    }
    fn foo$f1(input$l1: u8) -> u32 {
        (input$l1 as u32)
    }
    ");
}

#[test]
fn multiple_trait_impls_with_different_instantiations() {
    // Implement the same trait twice with different type parameters.
    let src = r#"
    trait MyTrait<T, U> {
        fn foo(left: T, right: T) -> U;
    }

    struct Foo<A, B>;

    impl MyTrait<u8, u8> for Foo<u8, u8> {
        fn foo(left: u8, right: u8) -> u8 {
            left + right
        }
    }

    impl MyTrait<u32, u64> for Foo<u32, u64> {
        fn foo(left: u32, right: u32) -> u64 {
            (left as u64) * (right as u64)
        }
    }

    pub fn main(x: u8, y: u32) -> pub u8 {
        let r1: u8 = Foo::<u8, u8>::foo(x, x);
        let _r2: u64 = Foo::<u32, u64>::foo(y, y);
        r1
    }
    "#;

    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    fn main$f0(x$l0: u8, y$l1: u32) -> pub u8 {
        let r1$l2 = foo$f1(x$l0, x$l0);
        let _r2$l3 = foo$f2(y$l1, y$l1);
        r1$l2
    }
    fn foo$f1(left$l4: u8, right$l5: u8) -> u8 {
        (left$l4 + right$l5)
    }
    fn foo$f2(left$l6: u32, right$l7: u32) -> u64 {
        ((left$l6 as u64) * (right$l7 as u64))
    }
    ");
}

#[test]
fn tuple_pattern_becomes_separate_params() {
    let src = r#"
    fn main() -> pub u32 {
        let ab = (1, 2);
        let cd = (3, 4);
        foo(ab, cd)
    }

    fn foo((a, b): (u32, u32), cd: (u32, u32)) -> u32 {
        a + b + cd.0 + cd.1
    }
    "#;

    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    fn main$f0() -> pub u32 {
        let ab$l0 = (1, 2);
        let cd$l1 = (3, 4);
        foo$f1(ab$l0, cd$l1)
    }
    fn foo$f1(a$l2: u32, b$l3: u32, cd$l4: (u32, u32)) -> u32 {
        (((a$l2 + b$l3) + cd$l4.0) + cd$l4.1)
    }
    ");
}

#[test]
fn return_impl_trait_becomes_underlying_type() {
    let src = r#"
    trait Foo {}
    struct Bar { x: u32, y: Field }
    impl Foo for Bar {}

    fn foo_bar() -> impl Foo {
        Bar { x: 0, y: 0 }
    }

    fn main() {
        let _fb = foo_bar();
    }
    "#;

    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    fn main$f0() -> () {
        let _fb$l0 = foo_bar$f1()
    }
    fn foo_bar$f1() -> (u32, Field) {
        {
            let x$l1 = 0;
            let y$l2 = 0;
            (x$l1, y$l2)
        }
    }
    ");
}

#[test]
fn unused_generic_becomes_field() {
    let src = r#"
    enum Foo<T> {
        A(T),
        B
    }
    fn main() {
        let _foo: Foo<u32> = Foo::B;
    }
    "#;

    // The enum is represented as (<index>, <variant-1-fields>, <variant-2-fields>)
    // Since variant-2 doesn't have a T value, even though we have u32 on the LHS it becomes Field.
    // FIXME(#11147): The mismatch between data and the type is rejected by the SSA validation.
    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    global B$g0: (Field, (Field,), ()) = (1, (0), ());
    fn main$f0() -> () {
        let _foo$l0 = B$g0
    }
    ");
}

#[test]
fn unused_const_generic_becomes_zero() {
    let src = r#"
    enum Foo<let N: u32> {
        A(str<N>),
        B
    }

    fn main() {
        let _f: Foo<5> = Foo::B;
    }
    "#;

    // The enum is represented as (<index>, <variant-1-fields>, <variant-2-fields>)
    // Since variant-2 doesn't use the N value, even though we have 5 on the LHS it becomes 0.
    // FIXME(#11146): The type vs data mismatch is rejected by the SSA validation.
    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r#"
    global B$g0: (Field, (str<5>,), ()) = (1, (""), ());
    fn main$f0() -> () {
        let _f$l0 = B$g0
    }
    "#);
}

#[test]
fn repeated_array() {
    let src = r#"
    fn main() {
        let _a = [1 + 2; 3];
    }
    "#;

    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    fn main$f0() -> () {
        let _a$l1 = {
            let repeated_element$l0 = (1 + 2);
            [repeated_element$l0, repeated_element$l0, repeated_element$l0]
        }
    }
    ");
}

#[test]
fn repeated_array_zero() {
    let src = r#"
    fn main() {
        let _a = @[foo(); 0];
    }
    fn foo() -> Field {
        1 + 2
    }
    "#;

    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    fn main$f0() -> () {
        let _a$l1 = {
            let repeated_element$l0 = foo$f1();
            @[]
        }
    }
    fn foo$f1() -> Field {
        (1 + 2)
    }
    ");
}

#[test]
fn trait_method() {
    let src = r#"
    struct Foo {
        a: u32,
    }

    trait Bar {
        fn bar(self, other: Self) -> bool;
    }

    impl Bar for Foo {
        fn bar(self, other: Self) -> bool {
            self.a == other.a
        }
    }

    fn main() -> pub bool {
        let f1 = Foo { a: 1 };
        let f2 = Foo { a: 2 };
        f1.bar(f2)
    }
    "#;

    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    fn main$f0() -> pub bool {
        let f1$l1 = {
            let a$l0 = 1;
            (a$l0)
        };
        let f2$l3 = {
            let a$l2 = 2;
            (a$l2)
        };
        bar$f1(f1$l1, f2$l3)
    }
    fn bar$f1(self$l4: (u32,), other$l5: (u32,)) -> bool {
        (self$l4.0 == other$l5.0)
    }
    ");
}

#[test]
fn infix_trait_method() {
    let src = r#"
    struct Foo {
        a: u32,
    }

    impl Eq for Foo {
        fn eq(self, other: Self) -> bool {
            self.a == other.a
        }
    }

    fn main() -> pub bool {
        let f1 = Foo { a: 1 };
        let f2 = Foo { a: 2 };
        f1 == f2
    }
    "#;

    let program = get_monomorphized_with_stdlib(src, stdlib_src::EQ).unwrap();

    insta::assert_snapshot!(program, @r"
    fn main$f0() -> pub bool {
        let f1$l1 = {
            let a$l0 = 1;
            (a$l0)
        };
        let f2$l3 = {
            let a$l2 = 2;
            (a$l2)
        };
        eq$f1(f1$l1, f2$l3)
    }
    fn eq$f1(self$l4: (u32,), other$l5: (u32,)) -> bool {
        (self$l4.0 == other$l5.0)
    }
    ");
}

#[test]
fn prefix_trait_method() {
    let src = r#"
    struct Foo {
        a: i32,
    }

    impl Neg for Foo {
        fn neg(self) -> Self {
            Self { a: -self.a }
        }
    }

    fn main() {
        let f1 = Foo { a: 1 };
        let _f2 = -f1;
    }
    "#;

    let program = get_monomorphized_with_stdlib(src, stdlib_src::NEG).unwrap();

    insta::assert_snapshot!(program, @r"
    fn main$f0() -> () {
        let f1$l1 = {
            let a$l0 = 1;
            (a$l0)
        };
        let _f2$l2 = neg$f1(f1$l1)
    }
    fn neg$f1(self$l3: (i32,)) -> (i32,) {
        {
            let a$l4 = (-self$l3.0);
            (a$l4)
        }
    }
    ");
}

#[test]

fn fail_to_call_enum_member_without_panic() {
    // The 'Unexpected Type::Error found during monomorphization' error doesn't occur
    // when running this code as a real source file because in actual Noir code we don't
    // run the monomorphizer if there were previous errors. Here we do want to do that to
    // ensure this panic does not re-emerge.
    let src = "
        enum Foo {
            A
        }

        fn main() {
            let foo: Foo = Foo::A;
            foo(foo);
            ^^^^^^^^ Expected a function, but found a(n) Foo
            ^^^^^^^^ Unexpected Type::Error found during monomorphization
        }

        fn foo(f: Foo) {
            let _ = f;
        }
    ";
    let features = vec![UnstableFeature::Enums];
    check_monomorphization_error_using_features(src, &features, true);
}

#[test]
fn lambda_pairs() {
    let src = r#"
    fn main() {
        constrained_context();
        // safety: test
        unsafe { unconstrained_context(); }
    }

    fn foo() {}
    unconstrained fn bar() {}

    fn baz(f: fn () -> ()) {
        f();
    }
    fn qux(f: unconstrained fn () -> ()) {
        // safety: test
        unsafe { f(); }
    }
    unconstrained fn quy(f: fn () -> ()) {
        f();
    }
    unconstrained fn quz(f: unconstrained fn () -> ()) {
        f();
    }

    fn constrained_context() {
        let f = foo; // should be (constrained, unconstrained), because we could call the constrained, or pass on the unconstrained
        let b = bar; // should be (unconstrained, unconstrained), because we only have unconstrained
        f();
        // safety: test
        unsafe {
            b();
        }
        baz(f);
        // baz(b);   // cannot pass unconstrained where constrained is expected
        qux(f);
        qux(b);
        // safety: test
        unsafe {
            quy(f);
            quy(b);
            quz(f);
            quz(b);
        }
    }

    unconstrained fn unconstrained_context() {
        let f = foo; // should be (unconstrained, unconstrained), because we can only call unconstrained
        let b = bar; // should be (unconstrained, unconstrained), because we only have unconstrained
        f();
        b();
        baz(f);
        // baz(b);   // cannot pass unconstrained where constrained is expected
        qux(f);
        qux(b);
        quy(f);
        quy(b);
        quz(f);
        quz(b);
    }
    "#;

    let program = get_monomorphized(src).unwrap();

    insta::assert_snapshot!(program, @r"
    fn main$f0() -> () {
        constrained_context$f1();;
        {
            unconstrained_context$f2();
        }
    }
    fn constrained_context$f1() -> () {
        let f$l0 = (foo$f3, foo$f4);
        let b$l1 = (bar$f5, bar$f5);
        f$l0.0();;
        {
            b$l1.0();
        };
        baz$f6(f$l0);;
        qux$f7(f$l0);;
        qux$f7(b$l1);;
        {
            quy$f8(f$l0);;
            quy$f8(b$l1);;
            quz$f9(f$l0);;
            quz$f9(b$l1);
        }
    }
    unconstrained fn unconstrained_context$f2() -> () {
        let f$l2 = (foo$f4, foo$f4);
        let b$l3 = (bar$f5, bar$f5);
        f$l2.1();;
        b$l3.1();;
        baz$f10(f$l2);;
        qux$f11(f$l2);;
        qux$f11(b$l3);;
        quy$f8(f$l2);;
        quy$f8(b$l3);;
        quz$f9(f$l2);;
        quz$f9(b$l3);
    }
    fn foo$f3() -> () {
    }
    unconstrained fn foo$f4() -> () {
    }
    unconstrained fn bar$f5() -> () {
    }
    fn baz$f6(f$l4: (fn() -> (), unconstrained fn() -> ())) -> () {
        f$l4.0();
    }
    fn qux$f7(f$l5: (fn() -> (), unconstrained fn() -> ())) -> () {
        {
            f$l5.0();
        }
    }
    unconstrained fn quy$f8(f$l6: (fn() -> (), unconstrained fn() -> ())) -> () {
        f$l6.1();
    }
    unconstrained fn quz$f9(f$l7: (fn() -> (), unconstrained fn() -> ())) -> () {
        f$l7.1();
    }
    unconstrained fn baz$f10(f$l8: (fn() -> (), unconstrained fn() -> ())) -> () {
        f$l8.1();
    }
    unconstrained fn qux$f11(f$l9: (fn() -> (), unconstrained fn() -> ())) -> () {
        {
            f$l9.1();
        }
    }
    ");
}

#[test]
fn global_lambda_becomes_local() {
    let src = r#"
    global FOO: u32 = 1;
    global BAR: fn(u32) -> u32 = bar;
    global BAZ: u32 = BAR(2);

    fn main(x: u32) -> pub u32 {
        let f = BAR;
        f(x) + BAZ
    }

    fn bar(x: u32) -> u32 { x + FOO }
    "#;

    let program = get_monomorphized(src).unwrap();

    insta::assert_snapshot!(program, @r"
    global BAZ$g0: u32 = 3;
    global FOO$g1: u32 = 1;
    fn main$f0(x$l0: u32) -> pub u32 {
        let f$l1 = (bar$f1, bar$f2);
        (f$l1.0(x$l0) + BAZ$g0)
    }
    fn bar$f1(x$l2: u32) -> u32 {
        (x$l2 + FOO$g1)
    }
    unconstrained fn bar$f2(x$l3: u32) -> u32 {
        (x$l3 + FOO$g1)
    }
    ");
}

#[test]
fn match_missing_case_becomes_constrain() {
    let src = r#"
    enum Foo {
        A(u32),
        B
    }

    fn main() {
        let _ = foo(Foo::A(0));
    }

    fn foo(f: Foo) -> bool {
        match f {
            Foo::A(0) => true,
        }
    }
    "#;

    let program = get_monomorphized_with_options(
        src,
        GetProgramOptions {
            // Normally a missing case causes an elaboration failure,
            // which prevents monomorphization, but here we want to exercise
            // the code that would handle this, although it should never have to.
            allow_elaborator_errors: true,
            ..Default::default()
        },
    )
    .unwrap();

    insta::assert_snapshot!(program, @r#"
    fn main$f0() -> () {
        let _$l0 = foo$f1(A$f2(0))
    }
    fn foo$f1(f$l1: (Field, (u32,), ())) -> bool {
        {
            let internal variable$l2 = f$l1;
            match $2 {
                A($3) => match $3 {
                    0 => true,
                    _ => assert(false, "match failure"),
                },
                B => assert(false, "match failure"),
            }
        }
    }
    fn A$f2($0$l4: u32) -> (Field, (u32,), ()) {
        (0, ($0$l4), ())
    }
    "#);
}

#[test]
fn match_tuple_becomes_multiple_matches() {
    let src = r#"
    fn main(xy: (u32, u32)) -> pub bool {
        match xy {
            (0, _) => true,
            (_, 0) => true,
            _ => false,
        }
    }
    "#;

    let program = get_monomorphized(src).unwrap();

    insta::assert_snapshot!(program, @r"
    fn main$f0(xy$l0: (u32, u32)) -> pub bool {
        {
            let internal variable$l1 = xy$l0;
            match $1 {
                ($2, $3) => match $2 {
                    0 => {
                        let _$l4 = internal_match_variable_1$l3;
                        true
                    },
                    _ => match $3 {
                        0 => {
                            let _$l5 = internal_match_variable_0$l2;
                            true
                        },
                        _ => {
                            let _$l6 = internal variable$l1;
                            false
                        },
                    },
                },
            }
        }
    }
    ");
}

// Placeholder: code exists in the monomorphizer to handle `HirExpression::Guard`,
// but it looks like a guard is never constructed at the moment.
// When it is implemented, we should complete this test.
#[test]
#[should_panic(expected = "ParseError")]
fn match_guard_becomes_if_then_else() {
    let src = r#"
    fn main(xy: (u32, u32)) -> pub u32 {
        match xy {
            (x, y) if x == 0 => y,
            (x, _) => x,
        }
    }
    "#;

    let program = get_monomorphized(src).unwrap();

    insta::assert_snapshot!(program, @r"???");
}

#[test]
fn pass_ref_from_constrained_to_unconstrained_via_closure() {
    // The code below is invalid: it would result in passing a captured reference
    // as part of the environment from constrained to unconstrained environment.
    // However, it is not caught by monomorphization, but rather later by SSA validation.
    let src = r#"
    fn main()  {
        let mut x = 0;
        let f = foo(&mut x);
        f(1_u32);
        // safety: test
        unsafe { bar(f, 2_u32) }
    }

    fn foo(x: &mut u32) -> fn[(&mut u32,)](u32) -> () {
        |y| { *x = y; }
    }

    unconstrained fn bar<Env>(f: fn[Env](u32) -> (), x: u32) {
        f(x);
    }
    "#;

    let program = get_monomorphized(src).unwrap();

    insta::assert_snapshot!(program, @r"
    fn main$f0() -> () {
        let mut x$l0 = 0;
        let f$l1 = foo$f1((&mut x$l0));
        {
            let tmp$l2 = f$l1.0;
            tmp$l2.1(tmp$l2.0, 1)
        };;
        {
            bar$f2(f$l1, 2)
        }
    }
    fn foo$f1(x$l3: &mut u32) -> (((&mut u32,), fn(u32) -> () with closure environment (&mut u32,)), ((&mut u32,), unconstrained fn(u32) -> () with closure environment (&mut u32,))) {
        ({
            let closure_variable$l6 = {
                let env$l5 = (x$l3);
                (env$l5, lambda$f3)
            };
            closure_variable$l6
        }, {
            let closure_variable$l9 = {
                let env$l8 = (x$l3);
                (env$l8, lambda$f4)
            };
            closure_variable$l9
        })
    }
    unconstrained fn bar$f2(f$l10: (((&mut u32,), fn(u32) -> () with closure environment (&mut u32,)), ((&mut u32,), unconstrained fn(u32) -> () with closure environment (&mut u32,))), x$l11: u32) -> () {
        {
            let tmp$l12 = f$l10.1;
            tmp$l12.1(tmp$l12.0, x$l11)
        };
    }
    fn lambda$f3(mut env$l5: (&mut u32,), y$l4: u32) -> () {
        *env$l5.0 = y$l4
    }
    unconstrained fn lambda$f4(mut env$l8: (&mut u32,), y$l7: u32) -> () {
        *env$l8.0 = y$l7
    }
    ");
}

#[test]
fn pass_ref_from_constrained_to_unconstrained_via_arg() {
    let src = r#"
    fn main()  {
        // safety: test
        unsafe { foo(&mut 0); }
    }

    unconstrained fn foo(_x: &mut u32) {}
    "#;

    let err = get_monomorphized_with_options(
        src,
        GetProgramOptions { allow_elaborator_errors: true, ..Default::default() },
    )
    .expect_err("should fail to monomorphize");

    assert!(matches!(err, MonomorphizationError::ConstrainedReferenceToUnconstrained { .. }));
}

#[test]
fn pass_ref_from_unconstrained_to_unconstrained_via_return() {
    let src = r#"
    fn main()  {
        // safety: test
        unsafe {
            let _x = foo();
        }
    }

    unconstrained fn foo() -> &mut u32 {
        &mut 0
    }
    "#;

    let err = get_monomorphized_with_options(
        src,
        GetProgramOptions { allow_elaborator_errors: true, ..Default::default() },
    )
    .expect_err("should fail to monomorphize");

    assert!(matches!(err, MonomorphizationError::UnconstrainedReferenceReturnToConstrained { .. }));
}

#[test]
fn evaluates_builtin_zeroed() {
    let src = r#"
    fn main() {
        let _a: [(u32, str<3>); 2] = zeroed();
    }
    "#;

    let program = get_monomorphized_with_stdlib(src, stdlib_src::ZEROED).unwrap();

    // Note that the zeroed value of a `str<3>` is `"\0\0\0"`, which prints as "".
    insta::assert_snapshot!(program, @"\nfn main$f0() -> () {\n    let _a$l0 = [(0, \"\0\0\0\"), (0, \"\0\0\0\")]\n}");
}

#[test]
fn evaluates_builtin_zeroed_function() {
    let src = r#"
    fn main() {
        let _f: fn (u32, str<3>) -> [Field; 2] = zeroed();
    }
    "#;

    let program = get_monomorphized_with_stdlib(src, stdlib_src::ZEROED).unwrap();

    insta::assert_snapshot!(program, @r"
    fn main$f0() -> () {
        let _f$l4 = (zeroed_lambda$f1, zeroed_lambda$f2)
    }
    fn zeroed_lambda$f1(_$l0: u32, _$l1: str<3>) -> [Field; 2] {
        [0, 0]
    }
    unconstrained fn zeroed_lambda$f2(_$l2: u32, _$l3: str<3>) -> [Field; 2] {
        [0, 0]
    }
    ");
}

#[test]
fn evaluates_builtin_checked_transmute() {
    let src = r#"
    fn main() {
        let _a = foo([1, 2, 3]);
    }

    fn foo<let N: u32>(a: [u32; N]) -> [u32; 3] {
        checked_transmute(a)
    }
    "#;

    let program = get_monomorphized_with_stdlib(src, stdlib_src::CHECKED_TRANSMUTE).unwrap();

    insta::assert_snapshot!(program, @r"
    fn main$f0() -> () {
        let _a$l0 = foo$f1([1, 2, 3])
    }
    fn foo$f1(a$l1: [u32; 3]) -> [u32; 3] {
        {
            a$l1
        }
    }
    ");
}

#[test]
fn does_not_evaluate_aliased_functions() {
    let src = r#"
    fn main() -> pub Field {
        let a = 1;
        let f = checked_transmute;
        let b = f(a);
        b
    }
    "#;

    let program = get_monomorphized_with_stdlib(src, stdlib_src::CHECKED_TRANSMUTE).unwrap();

    // We are using `checked_transmute` as a function value, so monomorphization will create
    // proxies for it to forward the call, but note that this code would crash during SSA
    // generation, as there is no intrinsic `checked_transmute` function.
    insta::assert_snapshot!(program, @r"
    fn main$f0() -> pub Field {
        let a$l0 = 1;
        let f$l1 = (checked_transmute$f1, checked_transmute$f2);
        let b$l2 = f$l1.0(a$l0);
        b$l2
    }
    #[inline_always]
    fn checked_transmute_proxy$f1(p0$l0: Field) -> Field {
        checked_transmute$checked_transmute(p0$l0)
    }
    #[inline_always]
    unconstrained fn checked_transmute_proxy$f2(p0$l0: Field) -> Field {
        checked_transmute$checked_transmute(p0$l0)
    }
    ");
}

#[test]
fn evaluates_builtin_modulus_functions() {
    let src = r#"
    fn main() {
        let _ = modulus_num_bits();
        let _ = modulus_le_bits();
        let _ = modulus_be_bits();
        let _ = modulus_le_bytes();
        let _ = modulus_be_bytes();
    }
    "#;

    let program = get_monomorphized_with_stdlib(src, stdlib_src::MODULUS).unwrap();

    insta::assert_snapshot!(program, @r"
    fn main$f0() -> () {
        let _$l0 = 254;
        let _$l1 = @[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 1, 0, 0, 1, 1, 0, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 1, 1, 1, 0, 1, 0, 0, 1, 1, 1, 0, 1, 1, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 1, 0, 1, 1, 1, 1, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 1, 0, 1, 1, 1, 0, 1, 0, 0, 0, 0, 1, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 0, 1, 1, 0, 1, 1, 0, 1, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 1, 1, 1, 0, 1, 1, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 1, 0, 0, 0, 0, 1, 1, 1, 0, 1, 0, 0, 1, 1, 1, 0, 0, 1, 1, 1, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0, 1, 1];
        let _$l2 = @[1, 1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1, 1, 1, 0, 0, 1, 1, 1, 0, 0, 1, 0, 1, 1, 1, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 1, 1, 0, 1, 1, 1, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 1, 1, 0, 1, 1, 0, 1, 1, 0, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 1, 0, 0, 0, 0, 1, 0, 1, 1, 1, 0, 1, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 1, 1, 1, 1, 0, 1, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 1, 1, 0, 1, 1, 1, 0, 0, 1, 0, 1, 1, 1, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 1, 0, 1, 1, 0, 0, 1, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
        let _$l3 = @[1, 0, 0, 240, 147, 245, 225, 67, 145, 112, 185, 121, 72, 232, 51, 40, 93, 88, 129, 129, 182, 69, 80, 184, 41, 160, 49, 225, 114, 78, 100, 48];
        let _$l4 = @[48, 100, 78, 114, 225, 49, 160, 41, 184, 80, 69, 182, 129, 129, 88, 93, 40, 51, 232, 72, 121, 185, 112, 145, 67, 225, 245, 147, 240, 0, 0, 1]
    }
    ");
}

#[test]
fn does_not_evaluate_array_len() {
    let src = r#"
    fn main() -> pub u32 {
        let a = [1, 2, 3];
        a.len()
    }
    "#;

    let program = get_monomorphized_with_stdlib(src, stdlib_src::ARRAY_LEN).unwrap();

    // The evaluation of array_len has been moved to the SSA in #1736
    insta::assert_snapshot!(program, @r"
    fn main$f0() -> pub u32 {
        let a$l0 = [1, 2, 3];
        len$array_len(a$l0)
    }
    ");
}
