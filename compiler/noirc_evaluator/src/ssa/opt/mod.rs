//! This folder contains each optimization pass for the SSA IR.
//!
//! Each pass is generally expected to mutate the SSA IR into a gradually
//! simpler form until the IR only has a single function remaining with 1 block within it.
//! Generally, these passes are also expected to minimize the final amount of instructions.
mod array_set;
mod as_slice_length;
mod assert_constant;
mod bubble_up_constrains;
mod constant_folding;
mod defunctionalize;
mod die;
pub(crate) mod flatten_cfg;
mod inlining;
mod mem2reg;
mod rc;
mod remove_bit_shifts;
mod remove_enable_side_effects;
mod remove_if_else;
mod resolve_is_unconstrained;
mod runtime_separation;
mod simplify_cfg;
mod unrolling;
