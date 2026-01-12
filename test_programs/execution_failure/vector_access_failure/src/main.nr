fn main(x: Field, y: pub Field) {
    let mut vector = [0; 2];
    if x == y {
        vector = vector.push_back(y);
        vector = vector.push_back(x);
    } else {
        vector = vector.push_back(x);
    }
    // This constraint should fail as the vector length is 3 and the index is 3
    // The right hand side AND case ensures that the circuit inputs have not changed
    // and we always hit the else case in the if statement above.
    assert((vector[3] == 0) & (vector[2] != y));
}
