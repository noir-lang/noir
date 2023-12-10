use dep::std;

fn main(
    verification_key : [Field; 114], 
    proof : [Field; 94], 
    public_inputs : [Field; 1], 
    key_hash : Field, 
) -> pub [Field;16]{
    let input_aggregation_object = [0; 16];
    std::verify_proof(
        verification_key.as_slice(), 
        proof.as_slice(), 
        public_inputs.as_slice(), 
        key_hash, 
        input_aggregation_object
    )
}
