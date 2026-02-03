global G_A: (Field, bool, bool) = (1, true, true);
unconstrained fn main() -> pub (Field, bool) {
    let mut ctx_limit: u32 = 2;
    func_5(((&mut [-1]), (&mut false)), G_A, (&mut ctx_limit))
}
unconstrained fn func_5(
    a: (&mut [Field; 1], &mut bool),
    mut c: (Field, bool, bool),
    ctx_limit: &mut u32,
) -> (Field, bool) {
    if ((*ctx_limit) == 0_u32) {
        (a.0[0], *a.1)
    } else {
        *ctx_limit = ((*ctx_limit) - 1_u32);
        func_5(if (*a.1) { a } else { (a.0, (&mut c.1)) }, c, ctx_limit)
    }
}
