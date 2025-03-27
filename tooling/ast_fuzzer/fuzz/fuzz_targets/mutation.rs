//! Perform random equivalence mutations on the AST and check that the
//! execution result does not change.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|_data: &[u8]| {
    // fuzzed code goes here
});
