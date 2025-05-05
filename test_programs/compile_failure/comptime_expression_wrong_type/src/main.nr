fn main() -> pub i64 {
    let x: i64 = comptime {
        unsafe { func_1(1) }
    };
    x
}

unconstrained fn func_1(b: i32) -> i32 {
    b                                                                                                                                                             
}