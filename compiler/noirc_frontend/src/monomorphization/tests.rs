#![cfg(test)]
use crate::tests::get_program;

use super::{ast::Program, errors::MonomorphizationError, monomorphize};

pub fn get_monomorphized(src: &str) -> Result<Program, MonomorphizationError> {
    let (_parsed_module, mut context, errors) = get_program(src);
    assert!(
        errors.iter().all(|err| !err.is_error()),
        "Expected monomorphized program to have no errors before monomorphization, but found: {errors:?}"
    );

    let main = context
        .get_main_function(context.root_crate_id())
        .unwrap_or_else(|| panic!("get_monomorphized: test program contains no 'main' function"));

    monomorphize(main, &mut context.def_interner, false)
}

fn check_rewrite(src: &str, expected: &str) {
    let program = get_monomorphized(src).unwrap();
    assert!(format!("{}", program) == expected);
}

#[test]
fn bounded_recursive_type_errors() {
    // We want to eventually allow bounded recursive types like this, but for now they are
    // disallowed because they cause a panic in convert_type during monomorphization.
    let src = "
        fn main() {
            let _tree: Tree<Tree<Tree<()>>> = Tree::Branch(
                Tree::Branch(Tree::Leaf, Tree::Leaf),
                Tree::Branch(Tree::Leaf, Tree::Leaf),
            );
        }

        enum Tree<T> {
            Branch(T, T),
            Leaf,
        }";

    let error = get_monomorphized(src).unwrap_err();
    assert!(matches!(error, MonomorphizationError::RecursiveType { .. }));
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
        }

        type OptAlias<T> = Opt<T>;

        enum Opt<T> {
            Some(T),
            None,
        }";

    let error = get_monomorphized(src).unwrap_err();
    assert!(matches!(error, MonomorphizationError::RecursiveType { .. }));
}

#[test]
fn mutually_recursive_types_error() {
    let src = "
        fn main() {
            let _zero = Even::Zero;
        }

        enum Even {
            Zero,
            Succ(Odd),
        }

        enum Odd {
            One,
            Succ(Even),
        }";

    let error = get_monomorphized(src).unwrap_err();
    assert!(matches!(error, MonomorphizationError::RecursiveType { .. }));
}

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
    check_rewrite(src, expected_rewrite);
}
