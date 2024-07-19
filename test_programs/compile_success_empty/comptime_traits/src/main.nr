use std::ops::Neg;

fn main() {
    comptime
    {
        // impl Eq for Field
        assert(3 == 3);

        // impl<T, let N: u32> Default for [T; N] where T: Default
        // impl Default for Field
        let array = Default::default();

        // impl<T, let N: u32> Eq for [T; N] where T: Eq
        // impl Eq for Field
        assert([1, 2] != array);
    }
}

struct MyType {
    value: i32,
}

comptime impl Neg for MyType {
    comptime fn neg(self) -> Self {
        self
    }
}

fn neg_at_comptime() {
    comptime
    {
        let value = MyType { value: 1 };
        let _result = -value;
    }
}

