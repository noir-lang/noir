use dep::std;
use dep::std::collections::vec::Vec;

fn main(x: Field, y: pub Field) {
    let mut vector = Vec::new();

    std::assert_eq(vector.len(), 0);
    for i in 0..5 {
        vector.push(i);
    }
    std::assert_eq(vector.len(), 5);
    for i in 0..5 {
        std::assert_eq(i, vector.get(i));
    }

    let last_elem = vector.pop();
    std::assert_eq(last_elem, 4);
    std::assert_eq(vector.len(), 4);

    vector.insert(2, 100);
    std::assert_eq(vector.get(2), 100);
    std::assert_eq(vector.get(4), 3);
    std::assert_eq(vector.len(), 5);

    let removed_elem = vector.remove(3);
    std::assert_eq(removed_elem, 2);
    std::assert_eq(vector.get(3), 3);
    std::assert_eq(vector.len(), 4);

    let mut inputs_vector = Vec::from_slice([x, y]);
    std::assert_eq(inputs_vector.get(0), x);
    std::assert_eq(inputs_vector.get(1), y);
}
