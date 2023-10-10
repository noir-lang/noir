// Issue 2911
fn main() {
    let mut slice : &mut [Field] = &mut [];
    slice = &mut (*slice).push_back(1);
    assert((*slice)[0] == 1);
}
