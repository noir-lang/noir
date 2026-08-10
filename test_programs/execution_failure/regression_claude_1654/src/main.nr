// The soundness half of the regression test for noir-lang/noir-claude#1654; see the matching
// program in `execution_success` for the mechanism.
//
// `arr[0]` is `1` for these inputs, so `out == 0` is false and execution must fail. When ACIR gen
// resolved the safe-index read on the taken branch as a disabled access it bound `out` to the
// constant `0` instead, and this program was accepted — a prover could satisfy a circuit whose
// source rejects the same witness. This is the case that separates a real fix from one that only
// moves the substitution elsewhere: a fix that zeroes the read differently still passes the
// `execution_success` half and fails here.
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
    assert(out == 0);
}
