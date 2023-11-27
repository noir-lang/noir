fn main() {
    let array: [Field; 3] = [1, 2, 3];
    assert(array.eq(array));
    // Ensure this still works if we have to infer the type of the integer literals
    let array = [1, 2, 3];
    assert(array.eq(array));
}

trait Eq {
    fn eq(self, other: Self) -> bool;
}

impl<T> Eq for [T; 3] where T: Eq {
    fn eq(self, other: Self) -> bool {
        let mut ret = true;
        for i in 0 .. self.len() {
            ret &= self[i].eq(other[i]);
        }
        ret
    }
}

impl Eq for Field {
    fn eq(self, other: Field) -> bool {
        self == other
    }
}
