pub fn main(i: u32, v: [u32; 5]) -> pub [[u32; 5]; 4] {
    let arr = [[1u32, 2, 3, 4, 5],[6, 7, 8, 9, 10],[11, 12, 13, 14, 15], [16, 17, 18, 19, 20]];
    let mut slice = arr.as_slice();
    slice = slice.insert(i, v);
    let (_, slice) = slice.pop_front();
    slice.as_array()
}
