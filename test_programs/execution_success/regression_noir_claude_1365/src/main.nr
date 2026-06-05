// Regression for noir-lang/noir-claude#1365.
//
// `while i == 4` with `i` starting at 0 never executes its body. Loop invariant code motion
// inferred bounds `[0, 5)` from the `eq` header (upper = rhs + 1) and treated the body as
// definitely executed, hoisting `assert(x == 1)` into the loop pre-header. That rejected the
// valid execution `x = 0`, which should skip the body and return `0` unchanged.
unconstrained fn main(x: pub u32) -> pub u32 {
    let mut i: u32 = 0;

    while i == 4 {
        assert(x == 1);
        i = i + 1;
    }

    x
}
