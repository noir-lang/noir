fn main() {
    let unsorted: [u8; 3] = [3,1,2];
    let sorted = unsorted.sort();
    assert(sorted[0] == 1);
    assert(sorted[1] == 2);
    assert(sorted[2] == 3);
}
