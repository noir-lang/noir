unconstrained fn main() -> pub u32 {
    let s1 = loop_with_negative_literal();
    let s2 = loop_with_u128_literal();
    assert_eq(s1, s2);
    s1
}

fn loop_with_negative_literal() -> u32 {
    let s: i64 = -5;
    let e: i64 = 10;
    let mut sum = 0;
    for _ in s..e {
        sum += 1;
    }
    sum
}

fn loop_with_u128_literal() -> u32 {
    let s: u128 = 170141183460469231731687303715884105715;
    let e: u128 = 170141183460469231731687303715884105730;
    let mut sum = 0;
    for _ in s..e {
        sum += 1;
    }
    sum
}
