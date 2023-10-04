fn main(x : Field, y : pub Field) {
    let mut slice = [0; 2];
    if x == y {
        slice = slice.push_back(y);
        slice = slice.push_back(x);
    } else {
        slice = slice.push_back(x);
    }

    slice = slice.insert(10, 100);
}