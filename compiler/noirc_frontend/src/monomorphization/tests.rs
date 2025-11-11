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
