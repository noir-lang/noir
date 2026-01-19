fn main() -> pub i8 {
   let a = comptime {
        // Saftety: test
        unsafe {
            func_1(-128_i8)
        }
    };
    a
}

unconstrained fn func_1(a: i8) -> i8 {
    (a / -1)
}