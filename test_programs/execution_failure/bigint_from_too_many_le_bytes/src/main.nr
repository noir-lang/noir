use std::bigint::{bn254_fq, BigInt};

// TODO(https://github.com/noir-lang/noir/issues/5580): decide whether this is desired behavior
//
// Fails at execution time:
//
// error: Assertion failed: 'Index out of bounds'
//    ┌─ std/cmp.nr:35:34
//    │
// 35 │             result &= self[i].eq(other[i]);
//    │                                  --------
//    │
//    = Call stack:
//      1. /Users/michaelklein/Coding/rust/noir/test_programs/compile_failure/bigint_from_too_many_le_bytes/src/main.nr:7:12
//      2. std/cmp.nr:35:34
// Failed assertion
fn main() {
    let bytes: [u8] = bn254_fq.push_front(0x00);
    let bigint = BigInt::from_le_bytes(bytes, bn254_fq);
    let result_bytes = bigint.to_le_bytes();
    assert(bytes == result_bytes.as_slice());
}
