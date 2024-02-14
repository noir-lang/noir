fn main(mut x: [Field; 10], index: u8) -> pub [Field; 10] {
    x[index] = 0;
    x
}