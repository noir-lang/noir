// docs:start:poseidon
use dep::std::hash::poseidon;
use dep::std::hash::poseidon2;

fn main(x1: [field; 2], y1: pub field, x2: [field; 4], y2: pub field, x3: [field; 4], y3: field) {
    let hash1 = poseidon::bn254::hash_2(x1);
    assert(hash1 == y1);

    let hash2 = poseidon::bn254::hash_4(x2);
    assert(hash2 == y2);

    let hash3 = poseidon2::Poseidon2::hash(x3, x3.len());
    assert(hash3 == y3);
}
// docs:end:poseidon
