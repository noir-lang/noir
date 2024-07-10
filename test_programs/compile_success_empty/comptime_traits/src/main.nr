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
