use crate::tests::{assert_no_errors, check_errors};

#[test]
fn indexing_array_with_default_numeric_type_does_not_produce_an_error() {
    let src = r#"
    fn main() {
        let index = 0;
        let array = [1, 2, 3];
        let _ = array[index];
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn indexing_array_with_u32_does_not_produce_an_error() {
    let src = r#"
    fn main() {
        let index: u32 = 0;
        let array = [1, 2, 3];
        let _ = array[index];
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn indexing_array_with_non_u32_produces_an_error() {
    let src = r#"
    fn main() {
        let index: Field = 0;
        let array = [1, 2, 3];
        let _ = array[index];
                      ^^^^^ Indexing arrays and slices must be done with `u32`, not `Field`
    }
    "#;
    check_errors(src);
}

#[test]
fn indexing_array_with_non_u32_on_lvalue_produces_an_error() {
    let src = r#"
    fn main() {
        let index: Field = 0;
        let mut array = [1, 2, 3];
        array[index] = 0;
              ^^^^^ Indexing arrays and slices must be done with `u32`, not `Field`
    }
    "#;
    check_errors(src);
}

#[test]
fn cannot_determine_array_type() {
    let src = r#"
    fn main() {
        let _ = [];
                ^^ Type annotation needed
                ~~ Could not determine the type of the array
    }
    "#;
    check_errors(src);
}

#[test]
fn cannot_determine_slice_type() {
    let src = r#"
    fn main() {
        let _ = &[];
                ^^^ Type annotation needed
                ~~~ Could not determine the type of the slice
    }
    "#;
    check_errors(src);
}

#[test]
fn mutable_reference_to_array_element_as_func_arg() {
    let src = r#"
    fn foo(x: &mut u32) {
        *x += 1;
    }
    fn main() {
        let mut state: [u32; 4] = [1, 2, 3, 4];
        foo(&mut state[0]);
                 ^^^^^^^^ Mutable references to array elements are currently unsupported
                 ~~~~~~~~ Try storing the element in a fresh variable first
        assert_eq(state[0], 2); // expect:2 got:1
    }
    "#;
    check_errors(src);
}
