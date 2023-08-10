fn main(xs : [u8; 3]) {
    let sorted = xs.sort();
    assert(sorted[0] == 1);
    assert(sorted[1] == 2);
    assert(sorted[2] == 3);
}
