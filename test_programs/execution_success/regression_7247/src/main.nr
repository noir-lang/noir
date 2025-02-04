fn main(
    a: u32,
    b: u32,
    c: u32,
) {
    /// Safety: Test
    let vb = unsafe { via_brillig(a) };
    let va = via_acir(a);
    assert_eq(vb, b);
    assert_eq(va, b);

    /// Safety: Test
    let bat = unsafe { brillig_apply_twice(a, times_ten) };
    assert_eq(bat, b);

    let mat = mixed_apply_thrice(a, times_ten);
    assert_eq(mat, c);
}

unconstrained fn via_brillig(a: u32) -> u32 {
    times100(a)
}

fn via_acir(a: u32) -> u32 {
    times100(a)
}

fn apply_twice(a: u32, f: fn(u32) -> u32) -> u32 {
    f(f(a))
}

fn times100(a: u32) -> u32 {
    apply_twice(a, |x| x * 10)
}

fn times_ten(x: u32) -> u32 {
    // Random `if` to trigger `EnableSideEffect` during flattening.
    if x > 1000 {
        x * 10 - 5
    } else {
        x * 10
    }
}

unconstrained fn brillig_apply_twice(a: u32, f: fn(u32) -> u32) -> u32 {
    f(f(a))
}

fn mixed_apply_thrice(a: u32, f: fn(u32) -> u32) -> u32 {
    /// Safety: Test
    let aa = unsafe { brillig_apply_twice(a, f) };
    let aaa = f(aa);
    aaa
}
