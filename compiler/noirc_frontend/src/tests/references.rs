use crate::check_errors;

#[named]
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
    check_errors!(src);
}

#[named]
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
    check_errors!(src);
}

#[named]
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
    check_errors!(src);
}

#[named]
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
    check_errors!(src);
}
