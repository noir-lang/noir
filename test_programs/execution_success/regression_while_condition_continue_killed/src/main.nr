// The `continue` spelling of `regression_while_condition_break_killed`: a `continue` in a
// nested `while` condition also targets the OUTER loop and also clears the `killed` set,
// and that clear must survive the loop boundary just like for `break`.
//
// `i` is incremented before the inner `while`, so when the `continue` fires (`i == n`, the
// final iteration) it jumps to the outer condition, which is now false: the loop exits with
// the reassignment `x = [v, v, v]` skipped. The old value of `x` is still live after the
// loop, so `let mut y = x` must clone rather than move. The correct result is `[1, 9]`; the
// missing clone produces `[9, 9]` (or fails SSA validation in a debug build).
unconstrained fn main(v: Field, n: u32) -> pub [Field; 2] {
    let mut x = [v, v, v];
    let mut z = [0, 0, 0];
    let mut i = 0;
    while i < n {
        let mut y = x;
        y[0] = 9;
        z = y;
        i = i + 1;
        let mut j = 0;
        while (
            {
                if i == n {
                    continue;
                }
                j < 3
            }
        ) {
            j = j + 1;
        }
        x = [v, v, v];
    }
    [x[0], z[0]]
}
