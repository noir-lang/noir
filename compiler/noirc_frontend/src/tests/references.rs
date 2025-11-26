use crate::{
    elaborator::UnstableFeature,
    tests::{check_errors, get_program_using_features},
};

use super::assert_no_errors;

#[test]
fn cannot_mutate_immutable_variable() {
    let src = r#"
    fn main() {
        let array = [1];
        mutate(&mut array);
                    ^^^^^ Cannot mutate immutable variable `array`
    }

    fn mutate(_: &mut [Field; 1]) {}
    "#;
    check_errors(src);
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
                    ^^^^^ Cannot mutate immutable variable `foo`
    }

    fn mutate(foo: &mut Field) {
        *foo = 1;
    }
    "#;
    check_errors(src);
}

#[test]
fn mutable_reference_to_field_of_mutable_reference() {
    let src = r#"
    struct Foo {
        x: Field
    }
    
    fn main() {
        let mut foo = Foo { x: 5 };
        let ref_foo = &mut foo;
        let ref_x = &mut ref_foo.x;
        *ref_x = 10;
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn auto_dereferences_array_access() {
    let src = r#"
    fn main() {
        let mut ref_array = &mut &mut &mut [0, 1, 2];
        assert(ref_array[2] == 2);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn does_not_crash_when_passing_mutable_undefined_variable() {
    let src = r#"
    fn main() {
        mutate(&mut undefined);
                    ^^^^^^^^^ cannot find `undefined` in this scope
                    ~~~~~~~~~ not found in this scope
    }

    fn mutate(foo: &mut Field) {
        *foo = 1;
    }
    "#;
    check_errors(src);
}

#[test]
fn constrained_reference_to_unconstrained() {
    let src = r#"
    fn main(mut x: u32, y: pub u32) {
        let x_ref = &mut x;
        if x == 5  {
            // Safety: test context
            unsafe {
                mut_ref_input(x_ref, y);        
                              ^^^^^ Cannot pass a mutable reference from a constrained runtime to an unconstrained runtime
            }
        }

        assert(x == 10);
    }

    unconstrained fn mut_ref_input(x: &mut u32, y: u32) {
        *x = y;
    }
    "#;
    check_errors(src);
}

#[test]
fn immutable_references_with_ownership_feature() {
    let src = r#"
        unconstrained fn main() {
            let mut array = [1, 2, 3];
            borrow(&array);
        }

        fn borrow(_array: &[Field; 3]) {}
    "#;

    let (_, _, errors) = get_program_using_features(src, &[UnstableFeature::Ownership]);
    assert_eq!(errors.len(), 0);
}

#[test]
fn immutable_references_without_ownership_feature() {
    let src = r#"
        fn main() {
            let mut array = [1, 2, 3];
            borrow(&array);
                   ^^^^^^ This requires the unstable feature 'ownership' which is not enabled
                   ~~~~~~ Pass -Zownership to nargo to enable this feature at your own risk.
        }

        fn borrow(_array: &[Field; 3]) {}
                          ^^^^^^^^^^^ This requires the unstable feature 'ownership' which is not enabled
                          ~~~~~~~~~~~ Pass -Zownership to nargo to enable this feature at your own risk.
    "#;
    check_errors(src);
}
