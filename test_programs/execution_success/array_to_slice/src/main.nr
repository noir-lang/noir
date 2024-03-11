// Converts an array into a slice.
fn as_slice_push<T, N>(xs: [T; N]) -> [T] {
    let mut slice = [];
    for elem in xs {
        slice = slice.push_back(elem);
    }
    slice
}

fn main(x: Field, y: pub Field) {
    let xs: [Field; 0] = [];
    let ys: [Field; 1] = [1];
    let zs: [Field; 2] = [1, 2];
    let ws: [Field; 3] = [1; 3];
    let qs: [Field; 4] = [3, 2, 1, 0];

    assert(x != y);
    assert(xs.as_slice() == as_slice_push(xs));
    assert(ys.as_slice() == as_slice_push(ys));
    assert(zs.as_slice() == as_slice_push(zs));
    assert(ws.as_slice() == as_slice_push(ws));
    assert(qs.as_slice() == as_slice_push(qs));
}
