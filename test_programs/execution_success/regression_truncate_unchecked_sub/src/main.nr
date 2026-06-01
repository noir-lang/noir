fn main(x: pub u8) -> pub u32 {
    (256_u32 - (x as u32)) % 4
}
