fn main(x: [u32; 5], z: Field) {
    let idx = z + 10;
    // Dynamic index is greater than length of the array
    assert(x[idx] != 0);
}
