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
mod inlining;
mod mem2reg;
mod normalize_value_ids;
mod rc;
mod remove_bit_shifts;
mod remove_enable_side_effects;
mod remove_if_else;
mod resolve_is_unconstrained;
mod runtime_separation;
mod simplify_cfg;
mod unrolling;

#[cfg(test)]
pub(crate) fn assert_ssa_equals(mut ssa: super::Ssa, expected: &str) {
    ssa.normalize_ids();

    let ssa = ssa.to_string();
    let ssa = ssa.trim();
    let expected = expected.trim();

    if ssa != expected {
        println!("Expected:\n~~~\n{}\n~~~\nGot:\n~~~\n{}\n~~~", expected, ssa);
        similar_asserts::assert_eq!(expected, ssa);
    }
}
