// A read that follows an unconditional reassignment inside a loop body must still clone when
// another read precedes that reassignment.
//
// `h(x, i)` reads `x` after `x = [4, 5, 6]`, so nothing overwrites `x` again before the back edge
// reaches `g(x, i)` at the top of the next iteration. Moving `h`'s read lets `h` mutate that
// storage in place through its by-value parameter, and the next `g` reads the corrupted buffer.
// No `while` is involved: the loop's exit edge is irrelevant here, the back edge is what matters.
//
// Correct accumulation is `1 + 5 + 4 + 5 == 15`; a missing clone yields `1 + 5 + 99 + 5 == 110`.
// The trip count must come from a witness so the loop is not unrolled, and `h`'s index must too,
// so the in-place write survives to execution.
unconstrained fn main(i: u32, n: u32) {
    let mut x = [1, 2, 3];
    let mut acc = 0;
    for _j in 0..n {
        acc += g(x, i);
        x = [4, 5, 6];
        acc += h(x, i);
    }
    assert_eq(acc, 15);
}

unconstrained fn g(a: [u32; 3], _i: u32) -> u32 {
    a[0]
}

unconstrained fn h(mut a: [u32; 3], i: u32) -> u32 {
    a[i] = 99;
    a[1]
}
