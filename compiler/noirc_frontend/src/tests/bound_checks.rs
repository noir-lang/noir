use crate::hir::def_collector::dc_crate::CompilationError;

use super::get_program_errors;

#[test]
fn overflowing_u8() {
    let src = r#"
        fn main() {
            let _: u8 = 256;
        }"#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    if let CompilationError::TypeError(error) = &errors[0].0 {
        assert_eq!(
            error.to_string(),
            "The value `256` cannot fit into `u8` which has range `0..=255`"
        );
    } else {
        panic!("Expected OverflowingAssignment error, got {:?}", errors[0].0);
    }
}

#[test]
fn underflowing_u8() {
    let src = r#"
        fn main() {
            let _: u8 = -1;
        }"#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    if let CompilationError::TypeError(error) = &errors[0].0 {
        assert_eq!(
            error.to_string(),
            "The value `-1` cannot fit into `u8` which has range `0..=255`"
        );
    } else {
        panic!("Expected OverflowingAssignment error, got {:?}", errors[0].0);
    }
}

#[test]
fn overflowing_i8() {
    let src = r#"
        fn main() {
            let _: i8 = 128;
        }"#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    if let CompilationError::TypeError(error) = &errors[0].0 {
        assert_eq!(
            error.to_string(),
            "The value `128` cannot fit into `i8` which has range `-128..=127`"
        );
    } else {
        panic!("Expected OverflowingAssignment error, got {:?}", errors[0].0);
    }
}

#[test]
fn underflowing_i8() {
    let src = r#"
        fn main() {
            let _: i8 = -129;
        }"#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    if let CompilationError::TypeError(error) = &errors[0].0 {
        assert_eq!(
            error.to_string(),
            "The value `-129` cannot fit into `i8` which has range `-128..=127`"
        );
    } else {
        panic!("Expected OverflowingAssignment error, got {:?}", errors[0].0);
    }
}
