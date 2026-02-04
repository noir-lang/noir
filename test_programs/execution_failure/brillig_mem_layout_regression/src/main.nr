// This test is a minimized regression of `NOIR_AST_FUZZER_SEED=0xab2790100009e143 RUST_LOG=debug cargo test -p noir_ast_fuzzer_fuzz min_vs_full`
// If we do not appropriately set our stack and heap pointer this test is expected to fail with a type mismatch as we will be fetching 
// the programs internal type. Specifically for this test, if the free memory pointer is set too low the test will fail with a 
// type mismatch instead of a stack overflow.
unconstrained fn main(a: bool) -> pub i8 {
    let mut ctx_limit: u32 = 25;
    if a {
        if (
            if func_1(
                (
                    236405124039393412828585174085931355176,
                    &mut 65774936847330442457077268776944375055, 
                    (0, false, 82),
                    (156445504860752486204982473135107977804, 208),
                    [
                        1769493254999817056,
                        5821133370431581906,
                        2064486862708929924,
                        3330099083049573280,
                    ],
                ),
                (&mut ctx_limit),
            ) {
                0
            } else {
                0
            }
                == 186936179209777188558429361838459184562
        ) {
            55_i8
        } else {
            6_i8
        }
    } else {
        -110_i8
    }
}
unconstrained fn func_1(
    mut b: (Field, &mut u128, (u1, bool, u8), (Field, u8), [i64; 4]),
    ctx_limit: &mut u32,
) -> bool {
    if ((*ctx_limit) == 0_u32) {
        true
    } else if func_1(b, ctx_limit) {
        0 >= 0
    } else {
        false
    }
}
