//! ```text
//! cargo +nightly fuzz run init_vs_final -- -runs=10000 -max_len=1048576 -len_control=0
//! ```
#![no_main]

use libfuzzer_sys::arbitrary::Unstructured;
use libfuzzer_sys::fuzz_target;
use noir_ast_fuzzer_fuzz::targets::init_vs_final;

fuzz_target!(|data: &[u8]| {
    init_vs_final::fuzz(&mut Unstructured::new(data)).unwrap();
});
