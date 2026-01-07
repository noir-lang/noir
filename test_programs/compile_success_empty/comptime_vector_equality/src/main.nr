fn main() {
    comptime {
        assert_eq([1].as_vector(), [1].as_vector());
    }
}
