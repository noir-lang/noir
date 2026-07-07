fn main(mut arr: [u32; 3], idx: u32) -> pub [u32; 3] {
    let z: &mut [u32; 3] = &mut {
        let xx = &mut arr;
        *xx
    };
    let y = arr;
    (*z)[0] = 100;
    assert_eq((*z)[idx], 100);
    assert_eq(y, [1, 2, 3]);
    y
}
