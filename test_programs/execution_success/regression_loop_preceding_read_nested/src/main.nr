unconstrained fn main(i: u32, n: u32) -> pub u32 {
    let mut x = [1, 2, 3];
    let mut acc = 0;
    for _j in 0..n {
        for _m in 0..1 {
            acc += g(x);
        }
        x = [4, 5, 6];
        acc += h(x, i);
    }
    assert_eq(acc, 24);
    acc
}

unconstrained fn g(a: [u32; 3]) -> u32 {
    a[0]
}

unconstrained fn h(mut a: [u32; 3], i: u32) -> u32 {
    a[i] = 99;
    a[1]
}
