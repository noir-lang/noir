unconstrained fn inc_or_dec(c: bool) -> fn(u32) -> u32 {
    if c {
        |x| if x == 100 { 1000 } else { x + 1 }
    } else {
        |x| if x == 100 { 10 } else { x - 1 }
    }
}

fn main(c: bool, i: u32) -> pub u32 {
    let f =
        // safety: this cannot be constrained
        unsafe { inc_or_dec(c) };

    f(i)
}
