unconstrained fn main() -> pub bool {
    let e = func_5(&[(&mut false), (&mut false)]);
    let e = e.push_back((&mut false));
    let e = func_5(e);
    !(*e[0])
}
unconstrained fn func_5(a: [&mut bool]) -> [&mut bool] {
    if (*a[1]) {
        for idx_d in 0_u128..2_u128 {
            assert((if (1_u128 <= idx_d) { idx_d } else { idx_d } <= 0_u128), "FTN");
        }
        a
    } else {
        a
    }
}
