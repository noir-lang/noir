//! This folder contains each optimization pass for the SSA IR.
//!
//! Each pass is generally expected to mutate the SSA IR into a gradually
//! simpler form until the IR only has a single function remaining with 1 block within it.
//! Generally, these passes are also expected to minimize the final amount of instructions.

mod array_set;
mod as_slice_length;
mod assert_constant;
mod constant_folding;
mod defunctionalize;
mod die;
pub(crate) mod flatten_cfg;
mod hint;
mod inlining;
mod loop_invariant;
mod make_constrain_not_equal;
mod mem2reg;
mod normalize_value_ids;
mod preprocess_fns;
mod rc;
mod remove_bit_shifts;
mod remove_enable_side_effects;
mod remove_if_else;
mod remove_unreachable;
mod simplify_cfg;
mod unrolling;

/// Asserts that the given SSA, after normalizing its IDs and printing it,
/// is equal to the expected strings. Normalization is done so the IDs don't
/// shift depending on whether temporary intermediate values were created.
#[cfg(test)]
pub(crate) fn assert_normalized_ssa_equals(mut ssa: super::Ssa, expected: &str) {
    // First check if `expected` is valid SSA by parsing it, otherwise
    // the comparison will always fail but it won't be clear that it's because
    // expected is not valid.
    if let Err(err) = Ssa::from_str(expected) {
        panic!("`expected` argument of `assert_ssa_equals` is not valid SSA:\n{:?}", err);
    }

    use crate::{ssa::Ssa, trim_comments_from_lines, trim_leading_whitespace_from_lines};

    ssa.normalize_ids();

    let ssa = ssa.to_string();
    let ssa = trim_leading_whitespace_from_lines(&ssa);
    let expected = trim_leading_whitespace_from_lines(expected);
    let expected = trim_comments_from_lines(&expected);

    if ssa != expected {
        println!("Expected:\n~~~\n{expected}\n~~~\nGot:\n~~~\n{ssa}\n~~~");
        similar_asserts::assert_eq!(expected, ssa);
    }
}
