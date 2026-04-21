unconstrained fn main(mut arr: [u32; 3], idx: u32) {
    let z: &mut [u32; 3] = &mut arr;
    let y = arr;
    (*z)[idx] = 100;
    assert_eq((*z)[idx], 100);
    assert_eq(y[idx], 1);
}
