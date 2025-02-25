// Converts an array into a slice.
fn as_slice_push<T, let N: u32>(xs: [T; N]) -> [T] {
    let mut slice = &[];
    for elem in xs {
        slice = slice.push_back(elem);
    }
    slice
}

// Expected that x == 0 and y == 1
fn main(x: Field, y: pub Field) {
    let xs: [Field; 0] = [];
    let ys: [Field; 1] = [1];
    let zs: [Field; 2] = [1, 2];
    let ws: [Field; 3] = [1; 3];
    let qs: [Field; 4] = [3, 2, 1, 0];

    let mut dynamic: [Field; 4] = [3, 2, 1, 0];
    let dynamic_expected: [Field; 4] = [1000, 2, 1, 0];
    dynamic[x] = 1000;

    assert(x != y);
    assert(xs.as_slice() == as_slice_push(xs));
    assert(ys.as_slice() == as_slice_push(ys));
    assert(zs.as_slice() == as_slice_push(zs));
    assert(ws.as_slice() == as_slice_push(ws));
    assert(qs.as_slice() == as_slice_push(qs));

    assert(dynamic.as_slice()[0] == dynamic_expected[0]);
    assert(dynamic.as_slice()[1] == dynamic_expected[1]);
    assert(dynamic.as_slice()[2] == dynamic_expected[2]);
    assert(dynamic.as_slice()[3] == dynamic_expected[3]);
    assert(dynamic.as_slice().len() == 4);

    regression_4609_append_slices(x, y);
    regression_4609_append_dynamic_slices(x, y);
}

fn regression_4609_append_slices(x: Field, y: Field) {
    let sl = [x, 1, 2, 3].as_slice();
    let sl2 = [y, 5, 6].as_slice();
    let sl3 = sl.append(sl2);
    assert(sl3[0] == x);
    assert(sl3[4] == y);
}

fn regression_4609_append_dynamic_slices(x: Field, y: Field) {
    let mut sl = [x, 1, 2, 3].as_slice();
    sl[x] = x + 10;
    let mut sl2 = [y, 5, 6].as_slice();
    sl2[y] = y + 5;
    let sl3 = sl.append(sl2);
    assert(sl3[0] == 10);
    assert(sl3[4] == y);
    assert(sl3[5] == 6);
}
