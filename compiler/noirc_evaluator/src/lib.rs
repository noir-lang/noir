#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]

pub mod errors;

// SSA code to create the SSA based IR
// for functions and execute different optimizations.
pub mod ssa;

pub mod brillig;

pub use ssa::create_program;
