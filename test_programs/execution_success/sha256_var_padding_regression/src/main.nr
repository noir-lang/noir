// Test to check sha256_var produces same results irrespective of number of padding bytes after message.length
// Ref: https://github.com/noir-lang/noir/issues/6163, https://gist.github.com/jp4g/d5953faae9eadb2909357474f7901e58
fn main(preimage: [u8; 448], result: [u8; 32]) {
    // Construct arrays of different lengths
    let mut preimage_511 = [0; 511];
    let mut preimage_512 = [0; 512]; // Next block
    let mut preimage_575 = [0; 575];
    let mut preimage_576 = [0; 576]; // Next block
    for i in 0..preimage.len() {
        preimage_511[i] = preimage[i];
        preimage_512[i] = preimage[i];
        preimage_575[i] = preimage[i];
        preimage_576[i] = preimage[i];
    }
    let fixed_length_hash = std::hash::sha256::digest(preimage);
    let var_full_length_hash = std::hash::sha256::sha256_var(preimage, preimage.len() as u64);
    let var_length_hash_511 = std::hash::sha256::sha256_var(preimage_511, preimage.len() as u64);
    let var_length_hash_512 = std::hash::sha256::sha256_var(preimage_512, preimage.len() as u64);
    let var_length_hash_575 = std::hash::sha256::sha256_var(preimage_575, preimage.len() as u64);
    let var_length_hash_576 = std::hash::sha256::sha256_var(preimage_576, preimage.len() as u64);

    // All of the above should have produced the same hash (result)
    assert(fixed_length_hash == result);
    assert(var_full_length_hash == result);
    assert(var_length_hash_511 == result);
    assert(var_length_hash_512 == result);
    assert(var_length_hash_575 == result);
    assert(var_length_hash_576 == result);
}
