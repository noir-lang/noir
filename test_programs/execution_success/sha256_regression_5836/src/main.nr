fn main(headers: [u8; 472], result: pub [u8; 32]) {
    let hash = std::hash::sha256_var(headers, headers.len() as u64);
    assert_eq(hash, result);
}
