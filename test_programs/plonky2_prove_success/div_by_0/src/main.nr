fn main(x: u32, y: u32, z: u32) {
    let q = if (y != 0) {
        x / y
    } else {
        5
    };
    println(q);
    assert(q == z);
}
