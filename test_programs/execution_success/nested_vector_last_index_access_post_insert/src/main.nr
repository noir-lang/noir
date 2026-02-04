pub fn main(i: u32, v: (u32, [u32; 5])) -> pub u32 {
    let arr = [
        (1u32, [1u32, 2, 3, 4, 5]),
        (2, [6, 7, 8, 9, 10]),
        (3, [11, 12, 13, 14, 15]),
        (4, [16, 17, 18, 19, 20]),
    ];
    let mut vector = arr.as_vector();
    vector = vector.insert(i, v);
    vector = vector.insert(i, v);
    vector[5].0
}
