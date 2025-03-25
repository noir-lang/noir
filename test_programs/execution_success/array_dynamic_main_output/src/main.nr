fn main(mut x: [Field; 10], index: u8) -> pub [Field; 10] {
    x[index as u32] = 0;
    x
}
