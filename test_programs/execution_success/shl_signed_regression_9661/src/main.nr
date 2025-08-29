fn main() -> pub i8 {
    unsafe {
        foo(7_i8)
    }
}

unconstrained fn foo(a: i8) -> i8 {
    1_i8 << a;
    a
}
