fn main(mut a: pub Field) {
    let mut ctx_limit: u32 = 5_u32;
    if a == -1 {
        for _ in 0..2 {
            let d =
                // Safety: test
                unsafe { func_3_proxy(ctx_limit) } as Field;
            a = 1 / d;
        }
        for _ in 0..1 {
            assert(((a as u128) >= 1_u128));
        }
    }
}
unconstrained fn func_3(ctx_limit: &mut u32) -> bool {
    if ((*ctx_limit) == 0_u32) {
        false
    } else {
        *ctx_limit = ((*ctx_limit) - 1_u32);
        func_3(ctx_limit)
    }
}
unconstrained fn func_3_proxy(mut ctx_limit: u32) -> bool {
    func_3((&mut ctx_limit))
}
