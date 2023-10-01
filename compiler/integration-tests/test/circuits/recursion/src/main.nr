use dep::std;

fn main(
    verification_key : [Field; 114], 
    proof : [Field; 94], 
    public_inputs : [Field; 1], 
    key_hash : Field, 
    input_aggregation_object : [Field; 16],
) -> pub [Field;16]{
    let vk : [Field] = verification_key;
    let p : [Field] = proof;
    let pi : [Field] = public_inputs;
    std::verify_proof(
        vk, 
        p, 
        pi, 
        key_hash, 
        input_aggregation_object
    )
}
