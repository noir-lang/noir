#![cfg(test)]
use crate::{
    elaborator::UnstableFeature, test_utils::get_monomorphized,
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
    check_monomorphization_error_using_features(src, &features);
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
    check_monomorphization_error_using_features(src, &features);
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
    check_monomorphization_error_using_features(src, &features);
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
    check_monomorphization_error_using_features(src, &features);
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
#[should_panic(expected = "Type recursion limit reached - types are too large")]
fn extreme_type_alias_chain_stack_overflow() {
    // Generate a chain of 2,000 type aliases programmatically
    // ```
    // type Alias2000 = u8;
    // type Alias1999 = Alias2000;
    // type Alias1998 = Alias1999;
    // ...
    // type Alias1 = Alias2;
    // ```
    const DEPTH: usize = 2000;
    let mut aliases = String::new();

    // Start with the base type
    aliases.push_str(&format!("    type Alias{} = u8;\n", DEPTH));

    // Chain aliases from top to bottom
    for i in (1..DEPTH).rev() {
        aliases.push_str(&format!("    type Alias{} = Alias{};\n", i, i + 1));
    }

    // Insert the following alias chain:
    let src = format!(
        r#"
        {aliases}

        pub fn main(x: Alias1) -> pub u8 {{
            x
        }}
    "#
    );

    let _ = get_monomorphized(&src);
}
