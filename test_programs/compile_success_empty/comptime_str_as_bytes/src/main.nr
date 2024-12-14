fn main() {
    comptime {
        let hello_world_string = "hello world";
        let hello_world_bytes: [u8; 11] =
            [0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64];

        assert_eq(hello_world_string.as_bytes(), hello_world_bytes);
    }
}
