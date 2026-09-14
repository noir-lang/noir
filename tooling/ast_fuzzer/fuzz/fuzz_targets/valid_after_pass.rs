//! ```text
//! cargo +nightly fuzz run valid_after_pass -- -runs=10000 -max_len=1048576 -len_control=0
//! ```
#![no_main]

use libfuzzer_sys::arbitrary::Unstructured;
use libfuzzer_sys::fuzz_target;
use noir_ast_fuzzer_fuzz::targets::valid_after_pass;

fuzz_target!(|data: &[u8]| {
    valid_after_pass::fuzz(&mut Unstructured::new(data)).unwrap();
});
