global G_A: str<3> = "GOP";
global G_B: bool = true;
fn main(mut a: [i16; 4]) -> pub bool {
    let mut ctx_limit: u32 = (25_u32 / 1_u32);
    // Safety: fuzzer repro
    if unsafe {
        func_1_proxy(
            if (((a[0_u32] as i32) + 0_i32) > ((a[0_u32] as i32) + 0_i32)) {
                // Safety: fuzzer repro
                (unsafe { func_2_proxy(func_1, false, (ctx_limit / 1_u32)) }, G_A, "JID", ((false ^ (G_B ^ false)) as Field))
            } else {
                // Safety: fuzzer repro
                (unsafe { func_2_proxy(func_1, G_B, (ctx_limit + 0_u32)) }, G_A, G_A, ((G_B ^ (G_B ^ G_B)) as Field))
            },
            ctx_limit,
        )
    } {
        a = if (29618_u16 < 55293_u16) {
            a
        } else {
            a
        };
        // Safety: fuzzer repro
        if (!unsafe {
            func_1_proxy(
                if (152_u8 == 84_u8) {
                    ([a[3_u32]; 4], G_A, G_A, (G_B as Field))
                } else {
                    ([a[3_u32]; 4], G_A, G_A, (G_B as Field))
                },
                ctx_limit,
            )
        }) {
            for idx_e in 146_u8 .. 137_u8 {
                for idx_f in -1915219861_i32 ..= -1915219860_i32 {
                    let g: str<1> = "D";
                    ()
                };
                let mut h: ([i16; 4], str<3>, str<3>, Field) = (a, G_A, "SQI", (G_B as Field));
            };
        };
        G_B
    } else {
        G_B
    }
}
#[inline_always]
unconstrained fn func_1(a: ([i16; 4], str<3>, str<3>, Field), _ctx_limit: &mut u32) -> bool {
    (G_B | G_B)
}
#[inline_always]
unconstrained fn func_2(mut a: unconstrained fn(([i16; 4], str<3>, str<3>, Field), &mut u32) -> bool, mut b: bool, _ctx_limit: &mut u32) -> [i16; 4] {
    {
        for idx_c in (17783147942129234596539864047578952499_u128 % (8_u128 / 1_u128)) ..= (247774664374523177974012104361218220008_u128 % if b {
            8_u128
        } else {
            8_u128
        }) {
            {
                let mut idx_d: u32 = (3074668227_u32 - 3074668227_u32);
                while true {
                    if ((idx_d - 0_u32) == 3_u32) {
                        break
                    } else {
                        idx_d = (idx_d + 1_u32);
                        {
                            let mut idx_e: u32 = 0_u32;
                            loop {
                                if (idx_e == if b {
                                    5_u32
                                } else {
                                    5_u32
                                }) {
                                    break
                                } else {
                                    idx_e = ((idx_e + 1_u32) - 0_u32);
                                    assert(((334576937929521108774659452934584780550_u128 - 67540681797842494302423482174163720214_u128) <= (318477006065125437999926973285079636898_u128 - 2241553417313773954321835559821375364_u128)), "LGA");
                                }
                            }
                        };
                        println(idx_c);
                        {
                            let mut idx_f: u32 = (0_u32 - 0_u32);
                            while G_B {
                                if (idx_f == (6_u32 / 1_u32)) {
                                    break
                                } else {
                                    idx_f = (idx_f + 1_u32);
                                    for idx_g in 9396977227089442746_u64 ..= 9396977227089442744_u64 {
                                        a = func_1;
                                    };
                                    continue;
                                }
                            }
                        };
                        break;
                    }
                }
            };
            a = func_1;
        };
        [-19801_i16; 4]
    }
}
#[inline_always]
unconstrained fn func_1_proxy(a: ([i16; 4], str<3>, str<3>, Field), mut ctx_limit: u32) -> bool {
    func_1(a, (&mut ctx_limit))
}
#[inline_always]
unconstrained fn func_2_proxy(mut a: unconstrained fn(([i16; 4], str<3>, str<3>, Field), &mut u32) -> bool, mut b: bool, mut ctx_limit: u32) -> [i16; 4] {
    func_2(a, b, (&mut ctx_limit))
}
