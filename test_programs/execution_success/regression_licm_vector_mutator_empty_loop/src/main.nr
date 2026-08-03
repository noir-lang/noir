unconstrained fn main(x: bool, n: u32, m: u32) -> pub bool {
    let mut v = [x].as_vector();
    for _ in 0..n {
        v = v.push_back(x);
    }
    let mut acc = 0;
    for _ in 0..m {
        acc += v.push_front(false).len();
    }
    assert(acc == 0);
    assert(v[0] == x);
    v[0]
}
