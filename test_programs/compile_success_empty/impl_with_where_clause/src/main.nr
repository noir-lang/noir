fn main() {
    let array: [Field; 3] = [1, 2, 3];
    assert(array.my_eq(array));
    // Ensure this still works if we have to infer the type of the integer literals
    let array = [1, 2, 3];
    assert(array.my_eq(array));

    let other_array = [3, 2, 1];
    assert(array.my_ne(other_array));
}

trait MyEq {
    fn my_eq(self, other: Self) -> bool;

    fn my_ne(self, other: Self) -> bool{
        !MyEq::my_eq(self, other)
    }
}

impl<T> MyEq for [T; 3] where T: MyEq {
    fn my_eq(self, other: Self) -> bool {
        let mut ret = true;
        for i in 0 .. self.len() {
            ret &= self[i].my_eq(other[i]);
        }
        ret
    }
}

impl MyEq for Field {
    fn my_eq(self, other: Field) -> bool {
        self == other
    }
}
