// docs:start:pedersen-hash
fn main(x: Field, y: Field, expected_hash: Field) {
    let hash = std::hash::pedersen_hash([x, y]);
    assert_eq(hash, expected_hash);
}
// docs:end:pedersen-hash

