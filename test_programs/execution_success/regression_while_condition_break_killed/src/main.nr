// Regression for the `killed` set being restored across a loop boundary, discarding the
// `killed.clear()` performed by a `break` in a nested `while` condition.
//
// The `break` in the inner `while`'s condition targets the OUTER loop, so it can skip the
// reassignment `x = [v, v, v]`. The old value of `x` is then still live after the loop, so
// `let mut y = x` must clone rather than move: otherwise `y` aliases `x` and `y[0] = 9`
// corrupts the value read afterwards. `z = y` keeps the in-place `array_set` from being
// optimized away. With `v = 1` and `n = 5` the `break` fires on the last executed iteration
// (`i == 1`), so nothing repairs the corruption: the correct result is `[1, 9]`; the missing
// clone produces `[9, 9]` (or fails SSA validation in a debug build).
unconstrained fn main(v: Field, n: u32) -> pub [Field; 2] {
    let mut x = [v, v, v];
    let mut z = [0, 0, 0];
    let mut i = 0;
    while i < n {
        let mut y = x;
        y[0] = 9;
        z = y;
        let mut j = 0;
        while (
            {
                if i == 1 {
                    break;
                }
                j < 3
            }
        ) {
            j = j + 1;
        }
        x = [v, v, v];
        i = i + 1;
    }
    [x[0], z[0]]
}
