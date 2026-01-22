fn main(x: u32, y: pub u32, z: u32) {
    let mut result = x;
    for i in 1..11 {
        if i % 2 == 0 {
            result = result + y;
        } else {
            result = result - y;
        }
    }
    assert(result == z);
}
