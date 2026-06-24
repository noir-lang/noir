// Regression: a `while` condition that reads a variable also read in the body.
//
// A `while` evaluates its condition before the body, so the last use of `x` within an iteration
// is the body read, not the condition read. The ownership pass must therefore clone the condition
// read of `x` rather than move it. Here the condition aliases `x` into `y` and mutates `y`; if the
// condition read were moved (no clone), `y` would share `x`'s storage and the mutation would be
// observed by the body read of `x`. The correct result is `1 + 4 = 5`; the missing clone would
// instead produce `9 + 9 = 18` (or fail SSA validation in a debug build).
unconstrained fn main(n: u32) -> pub Field {
    let mut x = [1, 2, 3];
    let mut i = 0;
    let mut total = 0;
    while (
        {
            let mut y = x;
            y[0] = 9;
            i < n
        }
    ) {
        total = total + x[0];
        i = i + 1;
        x = [4, 5, 6];
    }
    total
}
