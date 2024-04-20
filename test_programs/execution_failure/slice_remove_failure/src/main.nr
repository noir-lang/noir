fn main(x: field, y: pub field) {
    let mut slice = [0; 2];
    if x == y {
        slice = slice.push_back(y);
        slice = slice.push_back(x);
    } else {
        slice = slice.push_back(x);
    }

    let (removed_slice, removed_elem) = slice.remove(10);
}
