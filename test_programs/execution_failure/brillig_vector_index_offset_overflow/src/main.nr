// Regression test for https://github.com/noir-lang/noir-claude/issues/1133
// A constant vector index near `u32::MAX` is shifted by the Brillig vector
// memory-layout offset (3) during the `brillig_array_get_and_set` pass. The
// shifted value must not overflow the 32-bit Brillig addressing type: the
// access should surface as a runtime out-of-bounds error rather than panicking
// in Brillig codegen.
//
// `get` is a separate unconstrained function taking an opaque vector, so its
// length is not known at the indexing site and the access is not folded into a
// compile-time out-of-bounds error before reaching the offset pass.
unconstrained fn get(v: [Field]) -> Field {
    v[4294967295]
}

fn main() -> pub Field {
    let v: [Field] = [1, 2, 3].as_vector();
    // Safety: testing an out-of-bounds vector access
    unsafe { get(v) }
}
