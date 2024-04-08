// docs:start:pedersen-commitment
use dep::std;

fn main(x: Field, y: Field, expected_commitment: std::hash::PedersenPoint) {
    let commitment = std::hash::pedersen_commitment([x, y]);
    assert_eq(commitment.x, expected_commitment.x);
    assert_eq(commitment.y, expected_commitment.y);
}
// docs:end:pedersen-commitment

