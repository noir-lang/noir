//! ```text
//! cargo +nightly fuzz run comptime_vs_brillig_direct
//! ```
#![no_main]

use libfuzzer_sys::arbitrary::Unstructured;
use libfuzzer_sys::fuzz_target;
use noir_ast_fuzzer_fuzz::targets::comptime_vs_brillig_direct;

fuzz_target!(|data: &[u8]| {
    comptime_vs_brillig_direct::fuzz(&mut Unstructured::new(data)).unwrap();
});
