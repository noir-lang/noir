#![cfg(test)]
use crate::{
    check_monomorphization_error_using_features,
    elaborator::UnstableFeature,
    test_utils::{Expect, get_monomorphized},
};

pub(crate) fn check_rewrite(src: &str, expected: &str, test_path: &str) {
    let program = get_monomorphized(src, test_path, Expect::Success).unwrap();
    assert!(format!("{}", program) == expected);
}

// NOTE: this will fail in CI when called twice within one test: test names must be unique
#[macro_export]
macro_rules! get_monomorphized {
    ($src:expr, $expect:expr) => {
        $crate::test_utils::get_monomorphized($src, $crate::function_path!(), $expect)
    };
}

// NOTE: this will fail in CI when called twice within one test: test names must be unique
#[macro_export]
macro_rules! check_rewrite {
    ($src:expr, $expected:expr) => {
        $crate::monomorphization::tests::check_rewrite($src, $expected, $crate::function_path!())
    };
}

#[named]
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
    check_monomorphization_error_using_features!(src, &features);
}

#[named]
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
    check_monomorphization_error_using_features!(src, &features);
}

#[named]
#[test]
fn mutually_recursive_types_error() {
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
    let features = vec![UnstableFeature::Enums];
    check_monomorphization_error_using_features!(src, &features);
}

#[named]
#[test]
fn simple_closure_with_no_captured_variables() {
    let src = r#"
    fn main() -> pub Field {
        let x = 1;
        let closure = || x;
        closure()
    }
    "#;

    let expected_rewrite = r#"fn main$f0() -> Field {
    let x$0 = 1;
    let closure$3 = {
        let closure_variable$2 = {
            let env$1 = (x$l0);
            (env$l1, lambda$f1)
        };
        closure_variable$l2
    };
    {
        let tmp$4 = closure$l3;
        tmp$l4.1(tmp$l4.0)
    }
}
fn lambda$f1(mut env$l1: (Field)) -> Field {
    env$l1.0
}
"#;
    check_rewrite!(src, expected_rewrite);
}
