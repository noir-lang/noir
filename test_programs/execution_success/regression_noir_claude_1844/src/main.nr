// Regression for noir-lang/noir-claude#1844.
//
// The body's first `if` becomes the loop header's `jmpif`, but both of its arms stay inside the
// loop: the loop actually exits through `i == 12` (or `i == x`) further down. The `i < 10`
// comparison therefore says nothing about how far `i` runs, and the compiler must not derive the
// induction-variable range `[0, 10)` from it. Doing so lets LICM fold the `i == 12` exit to
// `false` (12 is outside the bogus range), so the loop runs on to `i == x` and returns 99110
// instead of 310.
unconstrained fn f(x: u32) -> u32 {
    let mut i: u32 = 0;
    let mut c: u32 = 0;
    loop {
        if i < 10 {
            c += 1;
        } else {
            c += 100;
        }
        if i == 12 {
            break;
        }
        if i == x {
            break;
        }
        i += 1;
    }
    c
}

fn main(x: u32) -> pub u32 {
    // Safety: regression test for unconstrained loop shape
    unsafe {
        f(x)
    }
}
