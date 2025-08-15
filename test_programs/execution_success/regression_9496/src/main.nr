fn main(b: bool) -> pub u32 {
    let mut ctx_limit: u32 = 3_u32;
    // safety:
    unsafe {
        func_2_proxy(if { b } { &[10] } else { &[20, 30] }, ctx_limit)
    }
}
unconstrained fn func_2(a: [u32], ctx_limit: &mut u32) -> u32 {
    if *ctx_limit == 0 {
        a[1 % a.len()]
    } else {
        *ctx_limit = ((*ctx_limit) - 1_u32);
        let i = 3 % a.len();
        let b = a[i];
        let ab = a.push_back(b);
        println((a, i, b, ab));
        func_2(ab, ctx_limit)
    }
}
unconstrained fn func_2_proxy(a: [u32], mut ctx_limit: u32) -> u32 {
    func_2(a, (&mut ctx_limit))
}
