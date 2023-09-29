fn main(pair : (Field, Field), x: [(u8, u8, u8); 2]) -> pub (Field, Field) {
    assert(pair.0 == 1);
    assert(pair.1 == 0);

    let mut start_val = 0;
    for i in 0..2 {
        assert(x[i].0 == start_val);
        assert(x[i].1 == start_val + 1);
        assert(x[i].2 == start_val + 2);
        start_val += 3;
    }

    let (u, v) = if pair.0 as u32 < 1 {
        (pair.0, pair.0 + 1)
    } else {
        (pair.0 + 1, pair.0)
    };
    assert(u == pair.0 + 1);
    assert(v == pair.0);

    (u, v)
}