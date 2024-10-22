fn main(x: u32, y: pub u32) {
    for i in 0..10 {
        assert(x != i);
        assert(i != y);
    }
}
