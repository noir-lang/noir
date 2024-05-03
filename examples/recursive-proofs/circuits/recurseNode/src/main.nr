use dep::std;

fn main(
    verification_key: [Field; 114],
    public_inputs: pub [Field; 5],
    key_hash: Field,
    proof: [Field; 93] // 109 bytes were supposed to be required to verify recursive proofs (WIP)
) -> pub u64 {
    // verify sum was computed correctly
    std::verify_proof(
        verification_key.as_slice(),
        proof.as_slice(),
        public_inputs.as_slice(),
        key_hash
    );
    public_inputs[4] as u64
}
