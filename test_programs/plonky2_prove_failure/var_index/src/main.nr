fn main(x: [u32; 2], y: pub u32, z: pub u32) {
    assert(x[y] == z);
}
