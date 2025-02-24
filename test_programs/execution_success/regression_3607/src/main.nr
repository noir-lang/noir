fn main(mut x: u32) {
    if x == 0 {
        x = (x + 1) / x;
    } else {
        x = (x + 1) / x;
    }
    assert(x != 0);
}
