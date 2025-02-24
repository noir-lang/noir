global NUM_HASHES: u32 = 2;

fn main(foo: [u8; 95], toggle: bool, enable: [bool; NUM_HASHES]) {
    let mut result = [[0; 32]; NUM_HASHES];
    let mut const_result = [[0; 32]; NUM_HASHES];
    let size: Field = 93 + toggle as Field * 2;
    for i in 0..NUM_HASHES {
        if enable[i] {
            result[i] = std::sha256::sha256_var(foo, size as u64);
            const_result[i] = std::sha256::sha256_var(foo, 93);
        }
    }

    for i in 0..NUM_HASHES {
        assert_eq(result[i], const_result[i]);
    }
}
