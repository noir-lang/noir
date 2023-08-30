fn main() {
    // This should line should error as we will overflow the u32
    let b: u32 = 1 >> 32;
}
