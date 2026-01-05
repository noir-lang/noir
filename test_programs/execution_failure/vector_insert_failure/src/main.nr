fn main(x: Field, y: pub Field) {
    let mut vector = #[0; 2];
    if x == y {
        vector = vector.push_back(y);
        vector = vector.push_back(x);
    } else {
        vector = vector.push_back(x);
    }

    vector = vector.insert(10, 100);
}
