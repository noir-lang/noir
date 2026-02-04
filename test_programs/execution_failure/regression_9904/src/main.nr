fn main() -> pub i8 {
    // Safety:
    unsafe {
        func_1(-1_i8, false)
    }
}

unconstrained fn func_1(a: i8, b: bool) -> i8 {
    if b {
        1
    } else {
        if func_1(a, true) > 0 {
            (-128_i8 % a)
        } else {
            0
        }
    }
}
