// Regression: a `break` in a `while` condition targets the enclosing loop, not the `while`.
//
// The inner `while`'s condition unconditionally breaks, which exits the outer `while` before the
// assignment `x = if cond { ... }` commits. The old value of `x` is therefore still live after the
// loop, so `let mut y = x` must clone rather than move: otherwise `y` would alias `x` and
// `y[0] = 10` would corrupt the value read afterwards. `z = y` keeps the in-place `array_set` from
// being optimized away. The correct result is `[1, 10]`; the missing clone would produce `[10, 10]`
// (or fail SSA validation in a debug build).
unconstrained fn main(cond: bool) -> pub [Field; 2] {
    let mut x = [1, 2];
    let mut z = [0, 0];
    let mut i = 0;
    while i <= 3 {
        x = if cond {
            let mut y = x;
            y[0] = 10;
            z = y;
            let mut j = 0;
            while (
                {
                    break;
                    j < 3
                }
            ) {
                j = j + 1;
            }
            [4, 5]
        } else {
            x
        };
        i = i + 1;
    }
    [x[0], z[0]]
}
