fn main(x : Field, y : pub Field) {
    let mut slice = [0; 2];
    if x == y {
        slice = slice.push_back(y);
        slice = slice.push_back(x);
    } else {
        slice = slice.push_back(x);
    }
    // This constraint should fail as the slice length is 3 and the index is 3
    // The right hand side AND case ensures that the circuit inputs have not changed
    // and we always hit the else case in the if statement above.
    assert((slice[3] == 0) & (slice[2] != y));
}
