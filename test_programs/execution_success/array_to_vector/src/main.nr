// Converts an array into a vector.
fn as_vector_push<T, let N: u32>(xs: [T; N]) -> [T] {
    let mut vector = [].as_vector();
    for elem in xs {
        vector = vector.push_back(elem);
    }
    vector
}

// Expected that x == 0 and y == 1
fn main(x: u32, y: pub u32) {
    let xs: [Field; 0] = [];
    let ys: [Field; 1] = [1];
    let zs: [Field; 2] = [1, 2];
    let ws: [Field; 3] = [1; 3];
    let qs: [Field; 4] = [3, 2, 1, 0];

    let mut dynamic: [Field; 4] = [3, 2, 1, 0];
    let dynamic_expected: [Field; 4] = [1000, 2, 1, 0];
    dynamic[x] = 1000;

    assert(x != y);
    assert(xs.as_vector() == as_vector_push(xs));
    assert(ys.as_vector() == as_vector_push(ys));
    assert(zs.as_vector() == as_vector_push(zs));
    assert(ws.as_vector() == as_vector_push(ws));
    assert(qs.as_vector() == as_vector_push(qs));

    assert(dynamic.as_vector()[0] == dynamic_expected[0]);
    assert(dynamic.as_vector()[1] == dynamic_expected[1]);
    assert(dynamic.as_vector()[2] == dynamic_expected[2]);
    assert(dynamic.as_vector()[3] == dynamic_expected[3]);
    assert(dynamic.as_vector().len() == 4);

    regression_4609_append_vectors(x, y);
    regression_4609_append_dynamic_vectors(x, y);
}

fn regression_4609_append_vectors(x: u32, y: u32) {
    let sl = [x, 1, 2, 3].as_vector();
    let sl2 = [y, 5, 6].as_vector();
    let sl3 = sl.append(sl2);
    assert(sl3[0] == x);
    assert(sl3[4] == y);
}

fn regression_4609_append_dynamic_vectors(x: u32, y: u32) {
    let mut sl = [x, 1, 2, 3].as_vector();
    sl[x] = x + 10;
    let mut sl2 = [y, 5, 6].as_vector();
    sl2[y] = y + 5;
    let sl3 = sl.append(sl2);
    assert(sl3[0] == 10);
    assert(sl3[4] == y);
    assert(sl3[5] == 6);
}
