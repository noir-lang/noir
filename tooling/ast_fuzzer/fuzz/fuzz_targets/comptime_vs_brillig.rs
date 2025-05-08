//! ```text
//! cargo +nightly fuzz run comptime_vs_brillig
//! ```
#![no_main]

use libfuzzer_sys::arbitrary::Unstructured;
use libfuzzer_sys::fuzz_target;
use noir_ast_fuzzer_fuzz::targets::comptime_vs_brillig;

fuzz_target!(|data: &[u8]| {
    comptime_vs_brillig::fuzz(&mut Unstructured::new(data)).unwrap();
});
