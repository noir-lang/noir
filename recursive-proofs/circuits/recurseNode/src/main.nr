use dep::std;

fn main(
    verification_key: [Field; 114],
    public_inputs: pub [Field; 6],
    key_hash: Field,
    proof: [Field; 109]
) -> pub u64 {
    // verify sum was computed correctly
    std::verify_proof(
        verification_key.as_slice(),
        proof.as_slice(),
        public_inputs.as_slice(),
        key_hash
    );
    public_inputs[5] as u64
}
