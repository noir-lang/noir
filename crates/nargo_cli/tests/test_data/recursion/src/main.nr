use dep::std;

fn main(
    verification_key : [Field; 114], 
    proof : [Field; 94], 
    public_input : Field, 
    key_hash : Field, 
    input_aggregation_object : [Field; 16]
) -> pub [Field; 16] {
    let output_aggregation_object = std::verify_proof(
        verification_key, 
        proof, 
        public_input, 
        key_hash, 
        input_aggregation_object
    );

    let mut output = [0; 16];
    for i in 0..16 {
        output[i] = output_aggregation_object[i];
    }
    output
}