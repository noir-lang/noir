global NUM_HASHES = 2;

fn main(foo: [u8; 95], toggle: bool, enable: [bool; NUM_HASHES]) -> pub [[u8; 32]; NUM_HASHES] {
    let mut result = [[0; 32]; NUM_HASHES];
    for i in 0..NUM_HASHES {
        let size: Field = 93 + toggle as Field * 2;
        if enable[i] {
            result[i] = std::sha256::sha256_var(foo, size as u64);
        }
    }

    result
}
