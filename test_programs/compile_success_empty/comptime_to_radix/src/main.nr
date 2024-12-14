fn main() {
    comptime {
        let le_bytes: [u8; 3] = 257.to_le_bytes();
        assert_eq(le_bytes, [1, 1, 0]);

        let be_bytes: [u8; 3] = 257.to_be_bytes();
        assert_eq(be_bytes, [0, 1, 1]);
    }
}
