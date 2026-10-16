// An array pushed into a vector is a copy: writing to the array afterwards
// must not change the vector's element.
fn main(
    seed: [[Field; 2]; 1],
    xs: [Field; 3],
    idx: [u32; 3],
    j: u32,
    a: [Field; 2],
    b: [[Field; 2]; 1],
    i: u32,
    k: u32,
) -> pub [Field; 2] {
    let mut row = [0; 2];
    let mut rows = seed.as_vector();
    for n in 0..3 {
        row[idx[n]] = xs[n];
        rows = rows.push_back(row);
    }
    row[idx[0]] = 1000;
    let sum = rows[j][0] + rows[j][1];
    assert_eq(sum, 5);
    assert_eq(row[0], 1000);

    let mut e = a;
    e[i] = 7;
    let v = b.as_vector().push_front(e);
    e[k] = 99;
    assert_eq(v[0][0], 1);
    assert_eq(e[0], 99);

    [sum, v[0][0]]
}
