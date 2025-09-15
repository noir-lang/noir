fn main() -> pub i8 {
    // Saftety: test
    let b = unsafe {
            func_1(-128_i8)
    };
    assert(b != b - 1);
   b
}

unconstrained fn func_1(a: i8) -> i8 {
    (a / -1)
}