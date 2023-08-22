// Tests arithmetic operations on integers
fn main()  {
    let x: u32 = 6;
    let y: u32 = 2;

    assert_eq((x + y), add(x, y));

    assert_eq((x - y), sub(x, y));

    assert_eq((x * y), mul(x, y));

    assert_eq((x / y), div(x, y));

    // TODO SSA => ACIR has some issues with i32 ops
    assert(check_signed_div(6, 2, 3));

    assert_eq(eq(1, 2), false);
    assert(eq(1, 1));

    assert_eq(lt(x, y), false);
    assert(lt(y, x));

    assert_eq((x & y), and(x, y));
    assert_eq((x | y), or(x, y));

    // TODO SSA => ACIR has some issues with xor ops

    assert(check_xor(x, y, 4));
    assert_eq((x >> y), shr(x, y));
    assert_eq((x << y), shl(x, y));
}

unconstrained fn add(x : u32, y : u32) -> u32 {
    x + y
}

unconstrained fn sub(x : u32, y : u32) -> u32 {
    x - y
}

unconstrained fn mul(x : u32, y : u32) -> u32 {
    x * y
}

unconstrained fn div(x : u32, y : u32) -> u32 {
    x / y
}

unconstrained fn check_signed_div(x: i32, y: i32, result: i32) -> bool {
    (x / y) == result
}

unconstrained fn eq(x : u32, y : u32) -> bool {
    x == y
}

unconstrained fn lt(x : u32, y : u32) -> bool {
    x < y
}

unconstrained fn and(x : u32, y : u32) -> u32 {
    x & y
}

unconstrained fn or(x : u32, y : u32) -> u32 {
    x | y
}

unconstrained fn check_xor(x : u32, y : u32, result: u32) -> bool {
    (x ^ y) == result
}

unconstrained fn shr(x : u32, y : u32) -> u32 {
    x >> y
}

unconstrained fn shl(x : u32, y : u32) -> u32 {
    x << y
}

