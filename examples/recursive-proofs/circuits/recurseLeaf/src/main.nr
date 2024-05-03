use dep::std;
use dep::sumLib::secretSum;

#[recursive]
fn main(
    verification_key: [Field; 114],
    public_inputs: pub [Field; 3],
    key_hash: Field,
    proof: [Field; 93],
    num: u64
) -> pub u64 {
    // verify sum so far was computed correctly
    std::verify_proof(
        verification_key.as_slice(),
        proof.as_slice(),
        public_inputs.as_slice(),
        key_hash
    );
    secretSum((public_inputs[2] as u64), num)
}
