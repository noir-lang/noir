//! ```text
//! cargo +nightly fuzz run pass_vs_prev -- -runs=10000 -max_len=1048576 -len_control=0
//! ```
#![no_main]

use libfuzzer_sys::arbitrary::Unstructured;
use libfuzzer_sys::fuzz_target;
use noir_ast_fuzzer_fuzz::targets::pass_vs_prev;

fuzz_target!(|data: &[u8]| {
    pass_vs_prev::fuzz(&mut Unstructured::new(data)).unwrap();
});
