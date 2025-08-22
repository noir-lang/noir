fn main(v1: u16, v2: bool) -> pub u8 {
    if v2 {
        (v1 - 257) as u8
    } else {
        0
    }
}
