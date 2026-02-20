use std::collections::unbounded_vec::UnboundedVec;

fn main(x: u32) {
    let mut vector: UnboundedVec<Field> = UnboundedVec::from_array([2, 3]);
    vector.set(x, 4);
    if x == 0 {
        vector.push_back(5);
    }
    assert(vector.get(x) == 4);
}
