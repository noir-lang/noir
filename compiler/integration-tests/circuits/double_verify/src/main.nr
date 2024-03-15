use dep::std;

fn main(
    verification_key: [Field; 114],
    proof: [Field; 93],
    proof2: [Field; 93],
    public_inputs: [Field; 1],
    key_hash: Field
) {
    std::verify_proof(
        verification_key.as_slice(),
        proof.as_slice(),
        public_inputs.as_slice(),
        key_hash
    );

    std::verify_proof(
        verification_key.as_slice(),
        proof2.as_slice(),
        public_inputs.as_slice(),
        key_hash
    )
}
