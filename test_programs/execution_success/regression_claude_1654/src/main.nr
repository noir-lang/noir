// Regression test for noir-lang/noir-claude#1654.
//
// `remove_enable_side_effects` is free to move a branch predicate past an `array_get` at a
// statically safe index, because such a read declares that its ACIR lowering does not consume the
// side-effects predicate. ACIR gen used to read that predicate anyway when deciding whether an
// array operation was disabled, so the always-failing `if` arm below — which pins its own
// predicate to a compile-time zero — made the *taken* arm's read resolve to the constant `0`.
//
// The whole read disappeared from the emitted ACIR along with the `!= 999` assertion written over
// it, and `assert(out == 1)` became unsatisfiable while `assert(out == 0)` was accepted (the
// second half is covered by the matching test in `execution_failure`).
//
// With `idx = 1` the write lands away from slot 0, so `arr[0]` is still `1`.
fn main(idx: u32, c: bool, val: Field) {
    let mut arr = [1, 2, 3, 4];
    arr[idx] = val;
    let mut out = 0;
    if c {
        assert(false, "bad branch");
    } else {
        out = arr[0];
        assert(out != 999);
    }
    assert(out == 1);
}
