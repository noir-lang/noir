fn main(input: [u8; 2], toggle: bool) {
    let size: Field = 1 + toggle as Field;
    assert(!toggle);

    let variable_sha = std::sha256::sha256_var(input, size as u64);
    let constant_sha = std::sha256::sha256_var(input, 1);

    assert_eq(variable_sha, constant_sha);
}
