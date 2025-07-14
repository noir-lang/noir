//! ```text
//! cargo +nightly fuzz run min_vs_full -- -runs=10000 -max_len=1048576 -len_control=0
//! ```
#![no_main]

use libfuzzer_sys::arbitrary::Unstructured;
use libfuzzer_sys::fuzz_target;
use noir_ast_fuzzer_fuzz::targets::min_vs_full;

fuzz_target!(|data: &[u8]| {
    min_vs_full::fuzz(&mut Unstructured::new(data)).unwrap();
});
