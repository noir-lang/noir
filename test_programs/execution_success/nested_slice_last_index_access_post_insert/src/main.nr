pub fn main(i: u32, v: (u32, [u32; 5])) -> pub u32 {
    let arr = [(1u32, [1u32, 2, 3, 4, 5]),(2, [6, 7, 8, 9, 10]),(3, [11, 12, 13, 14, 15]), (4, [16, 17, 18, 19, 20])];
    let mut slice = arr.as_slice();
    slice = slice.insert(i, v);
    slice = slice.insert(i, v);
    slice[5].0
}
