fn main() {
    let a: [[[[Field; 2]; 2]; 2]; 2] = [[[[1, 2], [3, 4]], [[5, 6], [7, 8]]], [[[1, 2], [3, 4]], [[5, 6], [7, 8]]]];
    assert(a.my_eq(a));
}

trait MyEq {
    fn my_eq(self, other: Self) -> bool;
}

impl<T> MyEq for [T; 2] where T: MyEq {
    fn my_eq(self, other: Self) -> bool {
        self[0].my_eq(other[0])
          & self[0].my_eq(other[0])
    }
}
// Impl for u32 but not Field
impl MyEq for u32 {
    fn my_eq(self, other: Self) -> bool {
        self == other
    }
}
