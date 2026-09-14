// Regression for https://github.com/noir-lang/noir-claude/issues/1436
//
// The assignment `x = if cond { let mut y = x; ...; break; ... } else { x }` reads
// `x` into `y` on the RHS. The frontend ownership pass used to treat that RHS read
// as a confirmed move of `x` (no clone), reasoning that the surrounding assignment
// kills the old value. But the `break` exits the loop before the assignment commits,
// so the old value of `x` is still live afterwards. Without the clone, `y[0] = 9`
// mutates `x`'s storage in place, corrupting the value observed after the loop.
//
// `z = y` keeps the in-place `array_set` from being optimized away, and returning both
// `x[0]` and `z[0]` observes the mutation: the correct output is `[1, 9]`, while the
// missing clone yields `[9, 9]` (or, in a debug build, the array_set RC invariant fires
// before execution).
unconstrained fn main(cond: bool, brk: bool) -> pub [Field; 2] {
    let mut x = [1];
    let mut z = [0];
    loop {
        x = if cond {
            let mut y = x;
            y[0] = 9;
            z = y;
            if brk {
                break;
            }
            [0]
        } else {
            x
        };
    }
    [x[0], z[0]]
}
