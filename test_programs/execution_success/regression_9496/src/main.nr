fn main(b: bool) -> pub (u32, u32, u32, u32) {
    // safety:
    let x1 = unsafe { push_0_get_1(if { b } { #[10] } else { #[20, 30] }) };
    // safety:
    let x2 = unsafe { pop_back(if { b } { #[10] } else { #[20, 30] }) };
    // safety:
    let x3 = unsafe { pop_front(if { b } { #[10] } else { #[20, 30] }) };
    // safety:
    let x4 = unsafe { pop_front(if { b } { #[10, 20, 30] } else { #[40, 50, 60] }) };
    (x1, x2, x3, x4)
}

unconstrained fn push_0_get_1(xs: [u32]) -> u32 {
    let xs = xs.push_back(xs[0]);
    println(xs);
    xs[1]
}

unconstrained fn pop_back(xs: [u32]) -> u32 {
    let (xs, x) = xs.pop_back();
    println(xs);
    x
}

unconstrained fn pop_front(xs: [u32]) -> u32 {
    let (x, xs) = xs.pop_front();
    println(xs);
    x
}
