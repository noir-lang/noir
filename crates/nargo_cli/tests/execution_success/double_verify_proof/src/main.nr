use dep::std;

fn main(
    verification_key : [Field; 114], 
    proof : [Field; 94], 
    public_inputs : [Field; 1], 
    key_hash : Field, 
    input_aggregation_object : [Field; 16],
    proof_b : [Field; 94],
) -> pub [Field; 16] {

    // let verification_key : [Field] = verification_key;
    // let proof : [Field] = proof;
    // let proof_b : [Field] = proof_b;
    // let public_inputs : [Field] = public_inputs;

    // let output_aggregation_object_a = std::verify_proof(
    //     verification_key, 
    //     proof, 
    //     public_inputs, 
    //     key_hash,
    //     input_aggregation_object
    // );

    // let output_aggregation_object = std::verify_proof(
    //     verification_key,
    //     proof_b,
    //     public_inputs,
    //     key_hash,
    //     output_aggregation_object_a
    // );

    let output_aggregation_object_a = std::verify_proof(
        verification_key.as_slice(), 
        proof.as_slice(), 
        public_inputs.as_slice(), 
        key_hash, 
        input_aggregation_object
    );

    let output_aggregation_object = std::verify_proof(
        verification_key.as_slice(),
        proof_b.as_slice(),
        public_inputs.as_slice(),
        key_hash,
        output_aggregation_object_a
    );
    
    let mut output = [0; 16];
    for i in 0..16 {
        output[i] = output_aggregation_object[i];
    }
    output
}
