unconstrained fn main() -> pub [[u8; 2]; 1] {
    let mut ctx_limit: u32 = 1_u32;
    func_1(&mut ctx_limit)
}
unconstrained fn func_1(ctx_limit: &mut u32) -> [[u8; 2]; 1] {
    if ((*ctx_limit) == 0_u32) {
        [[19_u8, 112_u8]]
    } else {
        *ctx_limit = ((*ctx_limit) - 1_u32);
        let a = func_1(ctx_limit);
        let mut idx_b: u32 = 0_u32;
        loop {
            if (idx_b == 0_u32) {
                break
            } else {
                idx_b = (idx_b + 1_u32);
                let i = a[0_u32][0_u32] as u32;
                let c = (a[0_u32][0_u32] / a[i][0_u32]);
            }
        }
        a
    }
}
