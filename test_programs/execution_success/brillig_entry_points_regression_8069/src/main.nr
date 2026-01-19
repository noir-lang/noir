fn main() -> pub bool {
    let ctx_depth = 5;
    // Safety: testing context
    let cond = unsafe { func_1(true, 1, ctx_depth) };
    let _ = if !cond {
        // Safety: testing context
        unsafe { func_2(1, true, ctx_depth) }[0]
    } else {
        0
    };
    false
}
unconstrained fn func_1(a: bool, b: i8, mut ctx_depth: u32) -> bool {
    if (ctx_depth == 0) {
        false
    } else {
        ctx_depth = (ctx_depth - 1);
        func_1(false, func_2((a as Field), false, ctx_depth)[3], ctx_depth)
    }
}
unconstrained fn func_2(mut a: Field, mut b: bool, mut ctx_depth: u32) -> [i8; 4] {
    if (ctx_depth == 0) {
        [6, 101, 92, 30]
    } else {
        ctx_depth = (ctx_depth - 1);
        func_2(a, func_1(b, 0, ctx_depth), ctx_depth)
    }
}
