fn main(a: pub bool, b: pub bool, c: bool) -> pub bool {
    if b & a & !c {
        // Safety: test code
        unsafe { func_2_proxy((c as Field)); };
    };

    // (!{ (!c) })
    (!(true ^ ((!c) ^ true)))
}

unconstrained fn func_2(b: Field, ctx_limit: u32) -> Field {
    if ((ctx_limit) != 0_u32) {
        let _ = func_2(b, ctx_limit - 1);
    }
    0
}
unconstrained fn func_2_proxy(b: Field) -> Field {
    func_2(b, 0)
}
