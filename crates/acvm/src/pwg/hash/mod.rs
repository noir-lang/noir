use std::collections::BTreeMap;

use acir::{circuit::gate::GadgetCall, native_types::Witness};
use blake2::{Blake2s, Digest};
use noir_field::FieldElement;
use sha2::Sha256;

pub fn blake2s(initial_witness: &mut BTreeMap<Witness, FieldElement>, gadget_call: &GadgetCall) {
    generic_hash_256::<Blake2s>(initial_witness, gadget_call)
}

pub fn sha256(initial_witness: &mut BTreeMap<Witness, FieldElement>, gadget_call: &GadgetCall) {
    generic_hash_256::<Sha256>(initial_witness, gadget_call)
}

fn generic_hash_256<D: Digest>(
    initial_witness: &mut BTreeMap<Witness, FieldElement>,
    gadget_call: &GadgetCall,
) {
    let mut hasher = D::new();

    // For each input in the vector of inputs, check if we have their witness assignments (Can do this outside of match, since they all have inputs)
    for input_index in gadget_call.inputs.iter() {
        let witness = &input_index.witness;
        let num_bits = input_index.num_bits;

        let witness_assignment = initial_witness.get(witness);
        let assignment = match witness_assignment {
            None => panic!("cannot find witness assignment for {:?}", witness),
            Some(assignment) => assignment,
        };

        // Although we have bits, we need to truncate to bytes as this is the smallest atomic unit
        // for byte based hash functions. Consequence: u4 is seen as u8
        let bytes = assignment.truncate_to_bytes(num_bits);
        hasher.update(bytes);
    }
    let result = hasher.finalize();

    // Now split the hash result into two 128 bits
    // and store lower and upper into two field elements
    // This behavior is only because the scalar field is 254 bits.
    // XXX: I guess for larger fields, we can make it one field element, but it would be a bit annoying to modify your code based on the field size.
    let (low_128_bytes, high_128_bytes) = result.split_at(16);
    assert_eq!(low_128_bytes.len(), 16);
    assert_eq!(high_128_bytes.len(), 16);

    let low_128_field = FieldElement::from_bytes(low_128_bytes);
    let high_128_field = FieldElement::from_bytes(high_128_bytes);

    assert_eq!(gadget_call.outputs.len(), 2);

    initial_witness.insert(gadget_call.outputs[0], low_128_field);
    initial_witness.insert(gadget_call.outputs[1], high_128_field);
}
