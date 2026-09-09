fn helper(v: [bool]) -> Field {
    let mut len = v.push_front(false).len() as Field;
    for _ in 0..10 {
        len = len * 3 + 1;
    }
    len
}

unconstrained fn main(x: bool, n: u32) -> pub bool {
    let mut v = [x].as_vector();
    for _ in 0..n {
        v = v.push_back(x);
    }
    let mut acc = 0;
    for _ in 0..2 {
        acc += helper(v);
    }
    assert(acc != 0);
    assert(helper(v) != 0);
    assert(v[0] == x);
    v[0]
}
