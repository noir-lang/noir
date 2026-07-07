fn main(active: bool, a: pub i16) -> pub i16 {
    if active {
        (a >> a)
    } else {
        0
    }
}
