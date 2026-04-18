use noirc_errors::CustomDiagnostic;

use crate::tests::{check_errors, get_program_errors};

#[test]
fn shift_width_mismatch_emits_primary_error() {
    let src = r#"
        fn main(x: u64, n: u8) -> pub u64 {
            x << n
            ^^^^^^ Integers must have the same bit width LHS is 64, RHS is 8
        }
    "#;
    check_errors(src);
}

#[test]
fn shift_width_mismatch_emits_cast_hint_note() {
    let src = r#"
        fn main(x: u64, n: u8) -> pub u64 {
            x << n
        }
    "#;
    let diags: Vec<CustomDiagnostic> =
        get_program_errors(src).iter().map(CustomDiagnostic::from).collect();

    let hint = diags.iter().flat_map(|d| &d.notes).find(|note| {
        note.contains("shift amount must have the same bit width") && note.contains("as u64")
    });
    assert!(
        hint.is_some(),
        "expected a shift-cast hint note mentioning `as u64`; got diagnostics: {:?}",
        diags.iter().map(|d| (d.message.clone(), d.notes.clone())).collect::<Vec<_>>()
    );
}

#[test]
fn signed_shift_width_mismatch_suggests_signed_cast() {
    let src = r#"
        fn main(x: i64, n: i8) -> pub i64 {
            x << n
        }
    "#;
    let diags: Vec<CustomDiagnostic> =
        get_program_errors(src).iter().map(CustomDiagnostic::from).collect();

    let hint = diags.iter().flat_map(|d| &d.notes).find(|note| note.contains("as i64"));
    assert!(
        hint.is_some(),
        "expected hint to suggest `as i64` for signed shift; got: {:?}",
        diags.iter().map(|d| (d.message.clone(), d.notes.clone())).collect::<Vec<_>>()
    );
}

#[test]
fn shift_width_match_is_accepted() {
    let src = r#"
        fn main(x: u64, n: u64) -> pub u64 {
            x << n
        }
    "#;
    assert!(get_program_errors(src).is_empty(), "expected no errors for matched-width shift");
}
