//! ```text
//! cargo +nightly fuzz run fmt_line_comments
//! ```
#![no_main]

use libfuzzer_sys::arbitrary::Unstructured;
use libfuzzer_sys::fuzz_target;
use noir_ast_fuzzer_fuzz::targets::fmt_line_comments;

fuzz_target!(|data: &[u8]| {
    fmt_line_comments::fuzz(&mut Unstructured::new(data)).unwrap();
});
