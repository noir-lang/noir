fn main(arr: [u32; 3], idx: u32) -> pub [u32; 3] {
    let x: [u32; 3] = arr;
    let z: &mut [u32; 3] = &mut {
        let xx = x;
        let yy = xx;
        yy
    };
    let y = x;
    (*z)[0] = 100;
    assert_eq((*z)[idx], 100);
    assert_eq(y, [1, 2, 3]);
    y
}
