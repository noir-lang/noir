// Swapping two arrays through a temporary, inside a loop. Every read in the swap is followed by
// a reassignment of the variable it read, so on names alone each is a last use. Moved, the swap
// lowers to loop-header parameters permuted across the back edge with no `inc_rc` among them, and
// `rc_invariant` — which reasons about SSA values rather than which names are dead — rejects the
// `array_set` below as a possible write through an alias of `a`.
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
