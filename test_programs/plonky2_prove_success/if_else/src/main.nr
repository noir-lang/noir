fn main(x: u32, y: pub u32) {
    if x * y % 3 == 0 {
        assert(y == 1000);
    } else {
        assert(y == 2000);
    }
}
