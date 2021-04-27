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

        let bytes = assignment.fetch_nearest_bytes(num_bits as usize);
        hasher.update(bytes);
    }
    let result = hasher.finalize();
    for i in 0..32 {
        initial_witness.insert(
            gadget_call.outputs[i],
            FieldElement::from_be_bytes_reduce(&[result[i]]),
        );
    }
}
