fn bitwise_and(a: [u8; 32], b: [u8; 32]) -> [u8; 32] {
    let mut result: [u8; 32] = [0 as u8; 32];

    for i in 0..32 {
        result[i as u8] = a[i as u8] & b[i as u8];
    }

    result
}

fn main(a: [u8; 32], b: pub [u8; 32], c: [u8; 32]) {
    assert(bitwise_and(a, b) == c);
}
