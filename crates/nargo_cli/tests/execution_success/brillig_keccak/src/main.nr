use dep::std;

// Tests a very simple program.
// 
// The features being tested is keccak256 in brillig
fn main(x: Field, result: [u8; 32]) {
    // We use the `as` keyword here to denote the fact that we want to take just the first byte from the x Field
    // The padding is taken care of by the program
    let digest = keccak256([x as u8], 1);
    assert_eq(digest, result);
    
    //#1399: variable meesage size 
    let message_size = 4;
    let hash_a = keccak256([1,2,3,4], message_size);
    let hash_b = keccak256([1,2,3,4,0,0,0,0], message_size);

    assert_eq(hash_a, hash_b);

    let message_size_big = 8;
    let hash_c = keccak256([1,2,3,4,0,0,0,0], message_size_big);

    assert(hash_a != hash_c);
}

unconstrained fn keccak256<N>(data: [u8; N], msg_len: u32) -> [u8; 32] {
    std::hash::keccak256(data, msg_len)
}
