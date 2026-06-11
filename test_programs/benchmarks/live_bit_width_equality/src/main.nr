fn main(a: pub u128, b: pub u128, c: pub u128, d: pub u128, e: pub u128) -> pub bool {
    let s0 = a + b;
    let s1 = s0 + c;
    let s2 = s1 + d;
    let total = s2 + e;
    assert(total == 500 as u128);

    (a < 501 as u128) & (s0 < 501 as u128) & (s1 < 501 as u128) & (s2 < 501 as u128)
}
