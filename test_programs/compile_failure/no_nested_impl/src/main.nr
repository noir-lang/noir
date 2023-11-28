fn main() {
    let a: [[[[Field; 2]; 2]; 2]; 2] = [[[[1, 2], [3, 4]], [[5, 6], [7, 8]]], [[[1, 2], [3, 4]], [[5, 6], [7, 8]]]];
    assert(a.eq(a));
}

trait Eq {
    fn eq(self, other: Self) -> bool;
}

impl<T> Eq for [T; 2] where T: Eq {
    fn eq(self, other: Self) -> bool {
        self[0].eq(other[0])
          & self[0].eq(other[0])
    }
}
// Impl for u32 but not Field
impl Eq for u32 {
    fn eq(self, other: Self) -> bool {
        self == other
    }
}
