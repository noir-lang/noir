// Regression for noir-lang/noir-claude#1844, checked-arithmetic consumer.
//
// The body's first `if` becomes the loop header's `jmpif`, but both of its arms stay inside the
// loop; the loop only exits once `c > limit`. The `i < 10` comparison therefore does not bound
// `i`, and LICM must not use the range `[0, 10)` to prove `i += 1` cannot overflow. With
// `limit = 2000` the `u8` counter reaches 255, so the checked increment must abort with
// `attempt to add with overflow` rather than wrap to 0.
unconstrained fn f(limit: u32) -> u32 {
    let mut i: u8 = 0;
    let mut c: u32 = 0;
    loop {
        if i < 10 {
            c += 1;
        } else {
            c += 4;
        }
        if c > limit {
            break;
        }
        i += 1;
    }
    c
}

fn main(limit: u32) -> pub u32 {
    // Safety: regression test for unconstrained loop shape
    unsafe {
        f(limit)
    }
}
