fn main(mut x1: [u32; 4], x2: [u32; 4], idx: pub u32, v: pub u32) {
    x1[idx] = v;
    assert(x1 == x2);
}
