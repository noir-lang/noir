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

    fn foo_bar() -> Bar {
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
    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r#"
    global B$g0: (Field, (str<5>,), ()) = (1, (""), ());
    fn main$f0() -> () {
        let _f$l0 = B$g0
    }
    "#);
}
