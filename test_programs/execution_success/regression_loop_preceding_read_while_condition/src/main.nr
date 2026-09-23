unconstrained fn main(i: u32, n: u32) -> pub u32 {
    let mut x = [1, 2, 3];
    let mut k = 0;
    let mut acc = 0;
    while cond(x, k, n, &mut acc) {
        x = [4, 5, 6];
        acc += h(x, i);
        k += 1;
    }
    assert_eq(acc, 28);
    acc
}

unconstrained fn cond(a: [u32; 3], k: u32, n: u32, acc: &mut u32) -> bool {
    *acc += a[0];
    k < n
}

unconstrained fn h(mut a: [u32; 3], i: u32) -> u32 {
    a[i] = 99;
    a[1]
}
