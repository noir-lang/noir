// Swapping two arrays through a temporary, inside a loop, rotates buffers across the back edge.
// `let t = a` and `a = c` are each followed by a reassignment of the variable they read, so name-level
// liveness alone says both reads are dead and moves them, leaving the whole rotation free of reference
// count traffic. The `array_set` below then operates on a buffer the analysis cannot show is unshared,
// and `rc_invariant` rejects the program: with no `inc_rc` on the rotation, the in-place mutation is
// reachable from `a` as well.
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
