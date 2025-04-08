//! This folder contains each optimization pass for the SSA IR.
//!
//! Each pass is generally expected to mutate the SSA IR into a gradually
//! simpler form until the IR only has a single function remaining with 1 block within it.
//! Generally, these passes are also expected to minimize the final amount of instructions.

mod array_set;
mod as_slice_length;
mod assert_constant;
mod basic_conditional;
mod brillig_array_gets;
pub(crate) mod brillig_entry_points;
mod check_u128_mul_overflow;
mod constant_folding;
mod defunctionalize;
mod die;
pub(crate) mod flatten_cfg;
mod hint;
pub(crate) mod inlining;
mod loop_invariant;
mod make_constrain_not_equal;
mod mem2reg;
mod normalize_value_ids;
mod preprocess_fns;
pub(crate) mod pure;
mod rc;
mod remove_bit_shifts;
mod remove_enable_side_effects;
mod remove_if_else;
mod remove_truncate_after_range_check;
mod remove_unreachable;
mod simplify_cfg;
mod unrolling;

/// Asserts that the given SSA, after normalizing its IDs and printing it,
/// is equal to the expected string. Normalization is done so the IDs don't
/// shift depending on whether temporary intermediate values were created.
#[cfg(test)]
pub(crate) fn assert_normalized_ssa_equals(mut ssa: super::Ssa, expected: &str) {
    use crate::{ssa::Ssa, trim_comments_from_lines, trim_leading_whitespace_from_lines};

    // Clean up the expected SSA a bit
    let expected = trim_leading_whitespace_from_lines(expected);
    let expected = trim_comments_from_lines(&expected);

    // First check if `expected` is valid SSA by parsing it, otherwise
    // the comparison will always fail but it won't be clear that it's because
    // expected is not valid.
    let mut expected_ssa = match Ssa::from_str(&expected) {
        Ok(ssa) => ssa,
        Err(err) => {
            panic!("`expected` argument of `assert_ssa_equals` is not valid SSA:\n{:?}", err)
        }
    };

    // We won't exactly compare `expected` against `ssa`:
    // we parse it, normalize it and turn it back into a string.
    // This allows us to use any names and not just `b0`, `b1`, `v0`, `v1`, etc.
    // which is what the SSA printer produces.
    expected_ssa.normalize_ids();

    ssa.normalize_ids();

    let ssa = ssa.to_string();
    let ssa = ssa.trim_end();

    let expected_ssa = expected_ssa.to_string();
    let expected_ssa = expected_ssa.trim_end();

    if ssa == expected_ssa {
        return;
    }

    if expected != expected_ssa {
        println!("Expected (before ID normalization):\n~~~\n{expected}\n~~~\n");
    }

    println!("Expected (after ID normalization):\n~~~\n{expected_ssa}\n~~~\n");
    println!("Got:\n~~~\n{ssa}\n~~~");
    similar_asserts::assert_eq!(expected_ssa, ssa);
}
