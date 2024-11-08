fn main() {
    let array: [Field; 3] = [1, 2, 3];
    assert(array.my_eq(array));
    // Ensure this still works if we have to infer the type of the integer literals
    let array = [1, 2, 3];
    assert(array.my_eq(array));
}

trait MyEq {
    fn my_eq(self, other: Self) -> bool;
}

impl<T> MyEq for [T; 3]
where
    T: MyEq,
{
    fn my_eq(self, other: Self) -> bool {
        let mut ret = true;
        for i in 0..self.len() {
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
