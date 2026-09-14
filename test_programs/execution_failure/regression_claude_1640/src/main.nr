// Regression test for noir-lang/noir-claude#1640: LICM applied the `while` loop's
// induction bounds to the sibling `for` loop, folding `i < 3` to true and
// satisfying this false assertion.
fn main(x: u32) {
    // Safety: pure computation over `x`.
    let t = unsafe { inner(x) };
    // `inner` returns 0 for every x != 7, so this assertion must fail.
    assert(t == 20, "t must be 20");
}

unconstrained fn inner(x: u32) -> u32 {
    let mut i: u32 = 0;
    let mut acc: u32 = 0;
    while i < 3 {
        if x == 7 {
            acc += 1;
        }
        i += 1;
    }
    // `i` is 3 here, so `i < 3` is false on every iteration below.
    let mut total: u32 = 0;
    for _k in 0..2 {
        total += (i < 3) as u32;
    }
    total * 10 + acc
}
