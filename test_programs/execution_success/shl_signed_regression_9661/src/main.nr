fn main() -> pub i8 {
    // Safety: testing context
    unsafe {
        foo(7_i8)
    }
}

unconstrained fn foo(a: i8) -> i8 {
    let _ = 1_i8 << a;
    a
}
