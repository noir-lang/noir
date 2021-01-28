use acir::OPCODE;
use acir::native_types::{Witness};
use acir::circuit::gate::GadgetCall;
use noir_field::FieldElement;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub struct GadgetCaller {}

impl GadgetCaller {
    pub fn solve_gadget_call<'a>(
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
        gadget_call: &GadgetCall,
    ) {
        match gadget_call.name {
            OPCODE::SHA256 => {
                // Deal with SHA256
                let mut hasher = Sha256::new();

                // 0. For each input in the vector of inputs, check if we have their witness assignments (Can do this outside of match, since they all have inputs)
                for input_index in gadget_call.inputs.iter() {
                    let witness = &input_index.witness;
                    let num_bits = input_index.num_bits;

                    let witness_assignment = initial_witness.get(witness);
                    let assignment = match witness_assignment {
                        None => panic!("Cannot find witness assignment for {:?}", witness),
                        Some(assignment) => assignment,
                    };

                    // Although we have bits, we need to truncate to bytes as this is the smallest atomic unit
                    // for SHA256. Consequence: u4 is seem as u8
                    let bytes = assignment.truncate_to_bytes(num_bits);
                    hasher.update(bytes);
                }
                let result = hasher.finalize();

                // Now split the SHA256 result into two 128 bits
                // and store lower and upper into two field elements
                // This behaviour is only because the scalar field is 254 bits.
                // XXX: I guess for larger fields, we can make it one field element, but it would be a bit annoying to modify your code based on the field size.
                let (low_128_bytes, high_128_bytes) = result.split_at(16);
                assert_eq!(low_128_bytes.len(), 16);
                assert_eq!(high_128_bytes.len(), 16);

                let low_128_field = FieldElement::from_bytes(low_128_bytes);
                let high_128_field = FieldElement::from_bytes(high_128_bytes);

                assert_eq!(gadget_call.outputs.len(), 2);

                initial_witness.insert(gadget_call.outputs[0].clone(), low_128_field);
                initial_witness.insert(gadget_call.outputs[1].clone(), high_128_field);
            }
            OPCODE::AES => {
                panic!("AES is not yet implemented")
            } 
        }
        // Iterate through standard library functions
    }
}
