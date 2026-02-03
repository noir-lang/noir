fn main() -> pub [[bool; 1]; 1] {
    let mut ctx_limit: u32 = 1;
    // Safety: test program
    unsafe {
        func_1([[true]], [[true]], ctx_limit)
    }
}
unconstrained fn func_1(
    a: [[bool; 1]; 1],
    mut b: [[bool; 1]; 1],
    mut ctx_limit: u32,
) -> [[bool; 1]; 1] {
    if (ctx_limit == 0) {
        [[true]]
    } else {
        b[0][0] = false;
        println(a);
        b = func_1(a, b, ctx_limit - 1);
        a
    }
}
