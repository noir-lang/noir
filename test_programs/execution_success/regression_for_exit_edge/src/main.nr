// A read inside a `for` body that follows an unconditional reassignment must still clone when the
// variable is read after the loop.
//
// A `for` is tested at its header, so the loop's exit edge leaves from there: once the body has run
// for the last time the back edge reaches the header and falls straight out to `acc += x[0]`,
// without crossing `x = [4, 5, 6]` again. Moving `h`'s read lets `h` write through its by-value
// parameter into the very storage that post-loop read observes. No `break` or `continue` is
// involved; the header's exit edge alone is what keeps the read live.
//
// Correct accumulation is `5 + 5 + 4 == 14`; a missing clone yields `5 + 5 + 99 == 109`.
// The trip count must come from a witness so the loop is not unrolled, and `h`'s index must too,
// so the in-place write survives to execution.
unconstrained fn main(i: u32, n: u32) {
    let mut x = [1, 2, 3];
    let mut acc = 0;
    for _j in 0..n {
        x = [4, 5, 6];
        acc += h(x, i);
    }
    acc += x[0];
    assert_eq(acc, 14);
}

unconstrained fn h(mut a: [u32; 3], i: u32) -> u32 {
    a[i] = 99;
    a[1]
}
