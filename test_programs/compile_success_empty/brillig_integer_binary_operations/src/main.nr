// Tests arithmetic operations on integers
fn main() {
    let x: u32 = 6;
    let y: u32 = 2;

    assert((x + y) == unsafe { add(x, y) });

    assert((x - y) == unsafe { sub(x, y) });

    assert((x * y) == unsafe { mul(x, y) });

    assert((x / y) == unsafe { div(x, y) });
    // TODO SSA => ACIR has some issues with i32 ops
    assert(unsafe { check_signed_div(6, 2, 3)});

    assert(unsafe { eq(1, 2) } == false);
    assert(unsafe { eq(1, 1) });

    assert(unsafe { lt(x, y) } == false);
    assert(unsafe { lt(y, x) });

    assert((x & y) == unsafe { and(x, y) });
    assert((x | y) == unsafe { or(x, y) });
    // TODO SSA => ACIR has some issues with xor ops
    assert(unsafe { check_xor(x, y, 4) });
    assert((x >> y) == unsafe { shr(x, y) });
    assert((x << y) == unsafe { shl(x, y) });
}

unconstrained fn add(x: u32, y: u32) -> u32 {
    x + y
}

unconstrained fn sub(x: u32, y: u32) -> u32 {
    x - y
}

unconstrained fn mul(x: u32, y: u32) -> u32 {
    x * y
}

unconstrained fn div(x: u32, y: u32) -> u32 {
    x / y
}

unconstrained fn check_signed_div(x: i32, y: i32, result: i32) -> bool {
    (x / y) == result
}

unconstrained fn eq(x: u32, y: u32) -> bool {
    x == y
}

unconstrained fn lt(x: u32, y: u32) -> bool {
    x < y
}

unconstrained fn and(x: u32, y: u32) -> u32 {
    x & y
}

unconstrained fn or(x: u32, y: u32) -> u32 {
    x | y
}

unconstrained fn check_xor(x: u32, y: u32, result: u32) -> bool {
    (x ^ y) == result
}

unconstrained fn shr(x: u32, y: u32) -> u32 {
    x >> y
}

unconstrained fn shl(x: u32, y: u32) -> u32 {
    x << y
}

