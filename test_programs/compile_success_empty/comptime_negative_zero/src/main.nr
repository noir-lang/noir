fn main() {
    comptime {
        assert_eq(-0, 0);
        assert_eq(-1 + 1, 0);
        assert_eq(-1 - -1, 0);
        assert_eq(0 * -1, 0);
        assert_eq(0 / -1, 0);
    }
}
