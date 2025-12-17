fn main(x: Field, y: pub Field) {
    let mut list = [0; 2];
    if x == y {
        list = list.push_back(y);
        list = list.push_back(x);
    } else {
        list = list.push_back(x);
    }
    // This constraint should fail as the list length is 3 and the index is 3
    // The right hand side AND case ensures that the circuit inputs have not changed
    // and we always hit the else case in the if statement above.
    assert((list[3] == 0) & (list[2] != y));
}
