struct Wrapper {
    arr: [u32; 3],
}

// Taking a reference to a struct field (`&mut w2.arr`) aliases the whole struct `w2`.
// A subsequent copy of `w2` into `y` must clone so that a write through `z` triggers
// copy-on-write and leaves `y` unchanged.
unconstrained fn main(arr: [u32; 3], idx: u32) {
    let mut w2 = Wrapper { arr };
    let z: &mut [u32; 3] = &mut w2.arr;
    let y = w2;
    (*z)[idx] = 100;
    assert_eq((*z)[idx], 100);
    assert_eq(y.arr[idx], 1);
}
