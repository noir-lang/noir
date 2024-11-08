#![cfg(test)]

use super::Ssa;

pub(crate) fn assert_ssa_equals(mut ssa: Ssa, expected: &str) {
    ssa.normalize_ids();

    let ssa = ssa.to_string();
    let ssa = ssa.trim();
    let expected = expected.trim();

    if ssa != expected {
        println!("Expected:\n~~~\n{}\n~~~\nGot:\n~~~\n{}\n~~~", expected, ssa);
        similar_asserts::assert_eq!(expected, ssa);
    }
}
