fn main(year: u32, min_age: u8) -> pub u32 {
    if min_age == 0 {
        (year - 2) / 4
    } else if year > 1 {
        (year - 2) / 4
    } else {
        0
    }
}
