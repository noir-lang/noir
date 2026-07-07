global G_B: ([bool; 1],) = ([false],);
unconstrained fn main() -> pub [[bool; 1]; 1] {
    let mut ctx_limit: u32 = 25_u32;
    let _ = func_3(([true],), [G_B.0], (&mut ctx_limit));
    [G_B.0]
}
unconstrained fn func_3(a: ([bool; 1],), mut c: [[bool; 1]; 1], ctx_limit: &mut u32) -> bool {
    if ((*ctx_limit) > 0_u32) {
        *ctx_limit = ((*ctx_limit) - 1_u32);
        if func_3(G_B, [a.0], ctx_limit) {
            c[0_u32] = {
                {
                    let mut idx_d: u32 = 0_u32;
                    loop {
                        if (idx_d == 3_u32) {
                            break
                        } else {
                            idx_d = (idx_d + 1_u32);
                            ()
                        }
                    }
                };
                a.0
            };
        }
    }
    true
}
