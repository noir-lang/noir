use std::collections::vec::Vec;

fn main(x: Field, y: pub Field) {
    let mut vector = Vec::new();

    assert(vector.len() == 0);
    for i in 0..5 {
        vector.push(i);
    }
    assert(vector.len() == 5);
    for i in 0..5 {
        assert(i == vector.get(i));
    }

    let last_elem = vector.pop();
    assert(last_elem == 4);
    assert(vector.len() == 4);

    vector.insert(2, 100);
    assert(vector.get(2) == 100);
    assert(vector.get(4) == 3);
    assert(vector.len() == 5);

    let removed_elem = vector.remove(3);
    assert(removed_elem == 2);
    assert(vector.get(3) == 3);
    assert(vector.len() == 4);

    let mut inputs_vector = Vec::from_slice(&[x, y]);
    assert(inputs_vector.get(0) == x);
    assert(inputs_vector.get(1) == y);
}
