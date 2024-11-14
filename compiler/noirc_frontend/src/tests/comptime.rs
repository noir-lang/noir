use crate::hir::{comptime::InterpreterError, def_collector::dc_crate::CompilationError};

use super::{assert_no_errors, get_program_errors};

#[test]
fn correctly_evaluates_passing_range_checks() {
    let src = r#"
    fn main() {
        comptime {
            2.assert_max_bit_size::<8>()
        }
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn correctly_evaluates_failing_range_checks() {
    let src = r#"
    fn main() {
        comptime {
            256.assert_max_bit_size::<8>()
        }
    }
    "#;
    let errors = get_program_errors(src);
    println!("{errors:?}");
    assert_eq!(errors.len(), 2);
    // The error being reported here doesn't seem ideal but the user output is more useful.
    assert!(matches!(
        errors[1].0,
        CompilationError::InterpreterError(InterpreterError::ErrorNodeEncountered { .. })
    ));
}
