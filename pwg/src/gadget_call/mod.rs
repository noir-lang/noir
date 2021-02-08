use acir::circuit::gate::GadgetCall;
use acir::native_types::Witness;
use acir::{circuit::gate::GadgetInput, OPCODE};
use aztec_backend::barretenberg_rs::Barretenberg;
use blake2::Blake2s;
use noir_field::FieldElement;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

use crate::merkle::MerkleTree;

// Note that the outputs for things like sha256 need to be computed
// as they may be used in later arithmetic gates

pub struct GadgetCaller;

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
                // This behavior is only because the scalar field is 254 bits.
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
            OPCODE::Blake2s => {
                // Deal with Blake2s
                let mut hasher = Blake2s::new();

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
                    // for blake2s. Consequence: u4 is seem as u8
                    let bytes = assignment.truncate_to_bytes(num_bits);
                    hasher.update(bytes);
                }
                let result = hasher.finalize();

                // Now split the SHA256 result into two 128 bits
                // and store lower and upper into two field elements
                // This behavior is only because the scalar field is 254 bits.
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
            OPCODE::MerkleRoot => {
                assert!(gadget_call.inputs.len() > 1);

                let num_of_leaves = gadget_call.inputs.len();
                let depth = log2(num_of_leaves);

                let mut tree = MerkleTree::new(depth);

                //Compute the root
                let mut root = FieldElement::zero();
                for (index, leaf_idx) in gadget_call.inputs.iter().enumerate() {
                    let leaf = match initial_witness.get(&leaf_idx.witness) {
                        None => panic!("Cannot find witness assignment for {:?}", leaf_idx),
                        Some(assignment) => assignment,
                    };

                    root = tree.update_leaf(index, *leaf);
                }

                initial_witness.insert(gadget_call.outputs[0].clone(), root);
            }
            OPCODE::MerkleMembership => {
                let mut inputs_iter = gadget_call.inputs.iter();

                let _root = inputs_iter.next().expect("expected a root");
                let root = input_to_value(initial_witness, _root);

                let _leaf = inputs_iter.next().expect("expected a leaf");
                let leaf = input_to_value(initial_witness, _leaf);

                let _index = inputs_iter.next().expect("expected an index");
                let index = input_to_value(initial_witness, _index);

                let hash_path: Vec<_> = inputs_iter
                    .map(|input| input_to_value(initial_witness, input))
                    .collect();
                let result = MerkleTree::check_membership(hash_path, root, index, leaf);

                initial_witness.insert(gadget_call.outputs[0].clone(), result);
            }
            OPCODE::SchnorrVerify => {
                // In barretenberg, if the signature fails, then the whole thing fails.
                //
                use aztec_backend::barretenberg_rs::Barretenberg;
                use std::convert::TryInto;

                let mut inputs_iter = gadget_call.inputs.iter();

                let _pub_key_x = inputs_iter
                    .next()
                    .expect("expected `x` component for public key");
                let pub_key_x = input_to_value(initial_witness, _pub_key_x).to_bytes();

                let _pub_key_y = inputs_iter
                    .next()
                    .expect("expected `y` component for public key");
                let pub_key_y = input_to_value(initial_witness, _pub_key_y).to_bytes();

                let pub_key_bytes: Vec<u8> = pub_key_x
                    .to_vec()
                    .into_iter()
                    .chain(pub_key_y.to_vec())
                    .collect();
                let pub_key: [u8; 64] = pub_key_bytes.try_into().unwrap();

                let mut signature = [0u8; 64];
                for i in 0..64 {
                    let _sig_i = inputs_iter.next().expect(&format!(
                        "signature should be 64 bytes long, found only {} bytes",
                        i
                    ));
                    let sig_i = input_to_value(initial_witness, _sig_i);
                    signature[i] = *sig_i.to_bytes().last().unwrap()
                }

                let mut message = Vec::new();
                while let Some(msg) = inputs_iter.next() {
                    let msg_i_field = input_to_value(initial_witness, msg);
                    let msg_i = *msg_i_field.to_bytes().last().unwrap();
                    message.push(msg_i);
                }

                let mut barretenberg = Barretenberg::new();

                let result = barretenberg.verify_signature(pub_key, signature, &message);
                if result != FieldElement::one() {
                    dbg!("signature has failed to verify");
                }

                initial_witness.insert(gadget_call.outputs[0].clone(), result);
            }
            OPCODE::Pedersen => {
                let inputs_iter = gadget_call.inputs.iter();

                let scalars: Vec<_> = inputs_iter
                    .map(|input| {
                        // XXX: Clone is not desirable. Remove on next refactor.
                        // Although it is just a memcpy
                        input_to_value(initial_witness, input).clone()
                    })
                    .collect();

                let mut barretenberg = Barretenberg::new();
                let result = barretenberg.compress_many(scalars);
                initial_witness.insert(gadget_call.outputs[0].clone(), result);
            }
        }
        // Iterate through standard library functions
    }
}

fn input_to_value<'a>(
    witness_map: &'a BTreeMap<Witness, FieldElement>,
    input: &GadgetInput,
) -> &'a FieldElement {
    match witness_map.get(&input.witness) {
        None => panic!("Cannot find witness assignment for {:?}", input),
        Some(assignment) => assignment,
    }
}

fn log2(x: usize) -> u32 {
    let x = x as u128;
    assert!(x.is_power_of_two());
    assert!(x > 0);
    let u128_num_bits = std::mem::size_of::<u128>() * 8;

    u128_num_bits as u32 - x.leading_zeros() - 1
}
