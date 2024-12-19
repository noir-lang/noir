use crate::hir::{
    def_collector::dc_crate::CompilationError, resolution::errors::ResolverError,
    type_check::TypeCheckError,
};

use super::get_program_errors;

#[test]
fn cannot_mutate_immutable_variable() {
    let src = r#"
    fn main() {
        let array = [1];
        mutate(&mut array);
    }

    fn mutate(_: &mut [Field; 1]) {}
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::TypeError(TypeCheckError::CannotMutateImmutableVariable { name, .. }) =
        &errors[0].0
    else {
        panic!("Expected a CannotMutateImmutableVariable error");
    };

    assert_eq!(name, "array");
}

#[test]
fn cannot_mutate_immutable_variable_on_member_access() {
    let src = r#"
    struct Foo {
        x: Field
    }

    fn main() {
        let foo = Foo { x: 0 };
        mutate(&mut foo.x);
    }

    fn mutate(foo: &mut Field) {
        *foo = 1;
    }
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::TypeError(TypeCheckError::CannotMutateImmutableVariable { name, .. }) =
        &errors[0].0
    else {
        panic!("Expected a CannotMutateImmutableVariable error");
    };

    assert_eq!(name, "foo");
}

#[test]
fn does_not_crash_when_passing_mutable_undefined_variable() {
    let src = r#"
    fn main() {
        mutate(&mut undefined);
    }

    fn mutate(foo: &mut Field) {
        *foo = 1;
    }
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::ResolverError(ResolverError::VariableNotDeclared { name, .. }) =
        &errors[0].0
    else {
        panic!("Expected a VariableNotDeclared error");
    };

    assert_eq!(name, "undefined");
}

#[test]
fn constrained_reference_to_unconstrained() {
    let src = r#"
    fn main(mut x: u32, y: pub u32) {
        let x_ref = &mut x;
        if x == 5  {
            unsafe {
            //@safety: test context
                mut_ref_input(x_ref, y);        
            }
        }

        assert(x == 10);
    }

    unconstrained fn mut_ref_input(x: &mut u32, y: u32) {
        *x = y;
    }
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::TypeError(TypeCheckError::ConstrainedReferenceToUnconstrained { .. }) =
        &errors[0].0
    else {
        panic!("Expected an error about passing a constrained reference to unconstrained");
    };
}
