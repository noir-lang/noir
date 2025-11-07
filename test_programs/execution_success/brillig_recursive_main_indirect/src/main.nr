unconstrained fn foo(x: u32, f: unconstrained fn(u32) -> bool) -> bool {
    if f(x - 1) & (x > 1) {
        f(x - 2)
    } else {
        x < 2
    }
}

unconstrained fn main(x: u32) -> pub bool {
    if x == 0 {
        true
    } else {
        foo(x, main)
    }
}
