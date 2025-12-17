fn main(x: Field, y: pub Field) {
    let mut list = &[0; 2];
    if x == y {
        list = list.push_back(y);
        list = list.push_back(x);
    } else {
        list = list.push_back(x);
    }

    list = list.insert(10, 100);
}
