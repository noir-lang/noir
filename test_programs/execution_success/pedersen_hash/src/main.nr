// docs:start:pedersen-hash
use dep::std;

fn main(x: field, y: field, expected_hash: field) {
    let hash = std::hash::pedersen_hash([x, y]);
    assert_eq(hash, expected_hash);
}
// docs:end:pedersen-hash

