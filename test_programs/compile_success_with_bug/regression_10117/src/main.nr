fn main(input: [u8; 1]) -> pub [u8; 32] {
    let fun = if input[0] == 0 {
        std::hash::blake2s
    } else {
        |_| [0_u8; 32]
    };
    fun(input)
}
