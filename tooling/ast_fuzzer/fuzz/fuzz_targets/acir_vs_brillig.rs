//! Compare the execution of random ASTs between the normal execution
//! vs when everything is forced to be Brillig.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|_data: &[u8]| {
    // fuzzed code goes here
});
