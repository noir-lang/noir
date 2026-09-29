// Swapping two arrays through a temporary, inside a loop. Every read in the swap is followed by
// a reassignment of the variable it read, so each is a move and the loop permutes the two buffers
// between loop-header parameters with no `inc_rc` among them. The parameters enter the loop
// distinct and the back edge only exchanges them, so `rc_invariant` must accept the in-place
// `array_set` below.
//
// `a` and `c` hold distinct buffers throughout, so three swaps leave `a` as the input `c` and `c` as
// the input `a`, and the increment lands on `c` alone: `a == [10, 20]` and `c == [2, 2]`.
// Inputs come from `Prover.toml` and the trip count is a witness, so nothing is constant-folded or
// unrolled before the reference counts are checked.
unconstrained fn main(a: [Field; 2], c: [Field; 2], n: u32) {
    let mut a = a;
    let mut c = c;
    for _j in 0..n {
        let t = a;
        a = c;
        c = t;
    }
    c[0] = c[0] + 1;
    assert_eq(a, [10, 20]);
    assert_eq(c, [2, 2]);
}
