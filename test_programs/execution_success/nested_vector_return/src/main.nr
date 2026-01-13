fn main(i: u32) -> pub [[u32; 5]; 4] {
    let mut arr =
        [[1u32, 2, 3, 4, 5], [6, 7, 8, 9, 10], [11, 12, 13, 14, 15], [16, 17, 18, 19, 20]];
    arr[i][1] = 100;
    let vector = arr.as_vector();
    vector.as_array()
}
