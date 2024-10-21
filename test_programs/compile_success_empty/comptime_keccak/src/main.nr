// Tests a very simple program.
//
// The features being tested is keccak256 in brillig
fn main() {
    comptime {
        let x = 0xbd;
        let result = [
            0x5a, 0x50, 0x2f, 0x9f, 0xca, 0x46, 0x7b, 0x26, 0x6d, 0x5b, 0x78, 0x33, 0x65, 0x19,
            0x37, 0xe8, 0x05, 0x27, 0x0c, 0xa3, 0xf3, 0xaf, 0x1c, 0x0d, 0xd2, 0x46, 0x2d, 0xca,
            0x4b, 0x3b, 0x1a, 0xbf,
        ];
        // We use the `as` keyword here to denote the fact that we want to take just the first byte from the x Field
        // The padding is taken care of by the program
        let digest = keccak256([x as u8], 1);
        assert(digest == result);
        //#1399: variable message size
        let message_size = 4;
        let hash_a = keccak256([1, 2, 3, 4], message_size);
        let hash_b = keccak256([1, 2, 3, 4, 0, 0, 0, 0], message_size);

        assert(hash_a == hash_b);

        let message_size_big = 8;
        let hash_c = keccak256([1, 2, 3, 4, 0, 0, 0, 0], message_size_big);

        assert(hash_a != hash_c);
    }
}

comptime fn keccak256<let N: u32>(data: [u8; N], msg_len: u32) -> [u8; 32] {
    std::hash::keccak256(data, msg_len)
}
