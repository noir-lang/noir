fn main() {
    let array: [Field; 3] = [1, 2, 3];
    assert(foo(array));

    // Ensure this still works if we have to infer the type of the integer literals
    let array = [1, 2, 3];
    assert(foo(array));
}

fn foo<T>(x: T) -> bool where T: Eq {
    x.eq(x)
}

trait Eq {
    fn eq(self, other: Self) -> bool;
}

impl<T, N> Eq for [T; N] where T: Eq {
    fn eq(self, other: Self) -> bool {
        let mut ret = true;
        for i in 0 .. self.len() {
            ret &= self[i].eq(other[i]);
        }
        ret
    }
}
