fn encrypt_me(encryption_fn: fn([u8; 4]) -> [u8; 32], input: [u8; 4]) -> [u8; 32] {
    encryption_fn(input)
}

fn another_hash(input: [u8; 4]) -> [u8; 32] {
    let mut output = [0; 32];
    for i in 0..4 {
        output[i] = input[i];
    }
    output
}

pub fn main(input: pub [u8; 4]) -> pub [u8; 32] {
    let fun = if input[0] == 0 {
        std::hash::blake2s
    } else {
        another_hash
    };
    let output = encrypt_me(fun, input);
    for byte in output {
        assert(byte <= 255);
    }
    output
}
