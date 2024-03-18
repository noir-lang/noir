unconstrained fn brillig_as_slice(x: Field) -> (u64, Field, Field) {
    let mut dynamic: [Field; 1] = [1];
    dynamic[x] = 2;
    assert(dynamic[0] == 2);

    let brillig_slice = dynamic.as_slice();
    assert(brillig_slice.len() == 1);

    (brillig_slice.len(), dynamic[0], brillig_slice[0])
}

fn main(x: Field) {
    let (slice_len, dynamic_0, slice_0) = brillig_as_slice(x);
    assert(slice_len == 1);
    assert(dynamic_0 == 2);
    assert(slice_0 == 2);
}

