//! Compare the execution of random ASTs between the initial SSA
//! (or as close as we can stay to the initial state)
//! and the fully optimized version.
//!
//! ```text
//! cargo +nightly fuzz run init_vs_final -- -runs=10000 -max_len=1048576 -len_control=0
//! ```
#![no_main]

use libfuzzer_sys::arbitrary::Unstructured;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    noir_ast_fuzzer_fuzz::targets::init_vs_final::fuzz(&mut Unstructured::new(data)).unwrap();
});
