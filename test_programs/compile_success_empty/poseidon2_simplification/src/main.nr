use std::hash::poseidon2;

fn main() {
    let digest = poseidon2::Poseidon2::hash([0], 1);
    let expected_digest = 0x2710144414c3a5f2354f4c08d52ed655b9fe253b4bf12cb9ad3de693d9b1db11;
    assert_eq(digest, expected_digest);
}
