// The `test_fuzz_target` test binary is rooted at this module, while each fuzz target
// binary is rooted at its own file and reaches this module as `crate::fuzz_lib`. The
// alias and the path attribute let the mutator, which is shared with those binaries and
// so spells its imports `crate::fuzz_lib::...`, compile under both roots.
#[cfg(test)]
extern crate self as fuzz_lib;
#[cfg(test)]
#[path = "../mutations/mod.rs"]
mod mutations;

pub(crate) mod block_context;
pub(crate) mod corpus;
pub(crate) mod ecdsa;
pub(crate) mod function_context;
pub(crate) mod fuzz_target_lib;
pub(crate) mod fuzzer;
pub(crate) mod initial_witness;
pub(crate) mod instruction;
pub(crate) mod options;
pub(crate) mod program_context;

#[cfg(test)]
mod tests;

pub(crate) const NUMBER_OF_VARIABLES_INITIAL: u32 = 7;
