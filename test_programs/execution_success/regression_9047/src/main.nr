fn main() -> pub str<2> {
    let mut b = 0_i8;
    if unsafe { func_1_proxy() } {
        b = 112;
        b = ((-b) * b);
    };
    "ok"
}
unconstrained fn func_1_proxy() -> bool {
    false
}
