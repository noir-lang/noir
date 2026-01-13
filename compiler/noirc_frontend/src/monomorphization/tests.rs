#![cfg(test)]
use crate::{
    elaborator::UnstableFeature,
    test_utils::{GetProgramOptions, get_monomorphized, get_monomorphized_with_options},
    tests::check_monomorphization_error_using_features,
};

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
    // There is no stdlib in these tests, so the definition is repeated here.
    pub trait Eq {
        fn eq(self, other: Self) -> bool;
    }

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

    let program = get_monomorphized_with_options(
        src,
        GetProgramOptions { root_and_stdlib: true, ..Default::default() },
    )
    .unwrap();

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
    // There is no stdlib in these tests, so the definition is repeated here.
    pub trait Neg {
        fn neg(self) -> Self;
    }

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

    let program = get_monomorphized_with_options(
        src,
        GetProgramOptions { root_and_stdlib: true, ..Default::default() },
    )
    .unwrap();

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
