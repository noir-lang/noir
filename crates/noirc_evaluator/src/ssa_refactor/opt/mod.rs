//! This folder contains each optimization pass for the SSA IR.
//!
//! Each pass is generally expected to mutate the SSA IR into a gradually
//! simpler form until the IR only has a single function remaining with 1 block within it.
//! Generally, these passes are also expected to minimize the final amount of instructions.
mod flatten_cfg;
mod inlining;
mod mem2reg;
mod simplify_cfg;
mod unrolling;
