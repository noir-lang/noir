//! Perform random equivalence mutations on the AST and check that the
//! execution result does not change.
//!
//! ```text
//! cargo +nightly fuzz run orig_vs_mutant
//! ```
#![no_main]

use libfuzzer_sys::arbitrary::Unstructured;
use libfuzzer_sys::fuzz_target;
use noir_ast_fuzzer_fuzz::targets::orig_vs_mutant;

fuzz_target!(|data: &[u8]| {
    orig_vs_mutant::fuzz(&mut Unstructured::new(data)).unwrap();
});
