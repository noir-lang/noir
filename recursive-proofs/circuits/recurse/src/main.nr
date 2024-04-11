use dep::std;

// recursive summation
#[recursive]
fn main(
    verification_key: [Field; 114],
    proof: [Field; 93],
    public_inputs: [Field; 1],
    key_hash: Field,
    sum: Field,
    n: Field
) -> pub u64 {
    if (key_hash != 0) {
        std::verify_proof(
            verification_key.as_slice(),
            proof.as_slice(),
            public_inputs.as_slice(),
            key_hash
        );
    }
    sum as u64 + n as u64
}
