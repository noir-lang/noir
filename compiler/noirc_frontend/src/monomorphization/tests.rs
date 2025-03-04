#![cfg(test)]
use crate::tests::get_program;

use super::{ast::Program, errors::MonomorphizationError, monomorphize};

fn get_monomorphized(src: &str) -> Result<Program, MonomorphizationError> {
    let (_parsed_module, mut context, errors) = get_program(src);
    assert!(
        errors.iter().all(|err| !err.is_error()),
        "Expected monomorphized program to have no errors before monomorphization, but found: {errors:?}"
    );

    let crate_id = context.root_crate_id();
    let main = context
        .get_main_function(crate_id)
        .unwrap_or_else(|| panic!("get_monomorphized: test program contains no 'main' function"));
    monomorphize(main, &mut context.def_interner, false)
}

#[test]
fn recursive_type_errors() {
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
