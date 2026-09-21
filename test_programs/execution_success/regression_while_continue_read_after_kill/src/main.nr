// A `continue` reaches the loop header, which can exit the loop, so a read the `continue` carries
// past a later reassignment must still clone.
//
// `h(x, i)` reads `x` after `x = [4, 5, 6]`. The `continue` below it skips `x = [7, 8, 9]`, so the
// value `h` consumed is the one the condition then falls out on and `acc += x[0]` reads. Moving
// that read lets `h` write `99` through its by-value parameter into that same storage.
//
// Correct accumulation is `5 + 5 + 4 == 14`; a missing clone yields `5 + 5 + 99 == 109`.
// The trip count must come from a witness so the loop is not unrolled, and `h`'s index must too,
// so the in-place write survives to execution.
unconstrained fn main(c: bool, i: u32, n: u32) {
    let mut x = [1, 2, 3];
    let mut k = 0;
    let mut acc = 0;
    while k < n {
        k += 1;
        x = [4, 5, 6];
        acc += h(x, i);
        if c {
            continue;
        }
        x = [7, 8, 9];
    }
    acc += x[0];
    assert_eq(acc, 14);
}

unconstrained fn h(mut a: [u32; 3], i: u32) -> u32 {
    a[i] = 99;
    a[1]
}
