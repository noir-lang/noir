fn main(a: u32, b: u32) {
    // This version unrolls into a number of additions
    assert_eq(loop(5, a), b);
    // This version simplifies into a single `constraint 50 == b`
    assert_eq(loop(5, 10), b);
    // This version should not simplify down to a single constraint,
    // it should treat 10 as opaque:
    assert_eq(loop(5, std::hint::black_box(10)), b);
}

fn loop(n: u32, k: u32) -> u32 {
    let mut sum = 0;
    for _ in 0..n {
        sum = sum + k;
    }
    sum
}
