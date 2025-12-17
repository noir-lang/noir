fn main(x: u32) {
    let mut list = &[2, 3];
    list[x] = 4;
    if x == 0 {
        list = list.push_back(5);
    }
    assert(list[x] == 4);
}
