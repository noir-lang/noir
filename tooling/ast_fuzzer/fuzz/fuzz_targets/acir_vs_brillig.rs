//! ```text
//! cargo +nightly fuzz run acir_vs_brillig
//! ```
#![no_main]

use libfuzzer_sys::arbitrary::Unstructured;
use libfuzzer_sys::fuzz_target;
use noir_ast_fuzzer_fuzz::targets::acir_vs_brillig;

fuzz_target!(|data: &[u8]| {
    acir_vs_brillig::fuzz(&mut Unstructured::new(data)).unwrap();
});
