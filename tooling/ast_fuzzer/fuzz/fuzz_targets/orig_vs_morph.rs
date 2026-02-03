//! Perform random equivalence mutations on the AST and check that the
//! execution result does not change.
//!
//! ```text
//! cargo +nightly fuzz run orig_vs_morph
//! ```
#![no_main]

use libfuzzer_sys::arbitrary::Unstructured;
use libfuzzer_sys::fuzz_target;
use noir_ast_fuzzer_fuzz::targets::orig_vs_morph;

fuzz_target!(|data: &[u8]| {
    orig_vs_morph::fuzz(&mut Unstructured::new(data)).unwrap();
});
