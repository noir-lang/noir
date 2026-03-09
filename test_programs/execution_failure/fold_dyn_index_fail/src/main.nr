fn main(mut x: [u32; 5], z: u32) {
    x[z] = 4;
    dynamic_index_check(x, z + 10);
}

#[fold]
fn dynamic_index_check(x: [u32; 5], idx: u32) {
    // Dynamic index is greater than length of the array
    assert(x[idx] != 0);
}
