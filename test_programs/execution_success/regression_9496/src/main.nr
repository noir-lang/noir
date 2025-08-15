fn main(b: bool) -> pub (u32, u32) {
    // safety:
    unsafe {
        func_2(if { b } { &[10] } else { &[20, 30] })
    }
}
unconstrained fn func_2(xs: [u32]) -> (u32, u32) {
    let xsx = xs.push_back(xs[0]);
    println(xsx);
    let (xs, x) = xs.pop_back();
    println((xs, x));
    (xsx[1], x)
}
