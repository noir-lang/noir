use super::merkle::{flatten_path, MerkleTree};
use crate::pwg::{self, input_to_value};
use acir::{circuit::gate::GadgetCall, native_types::Witness, OPCODE};
use aztec_backend::barretenberg_rs::Barretenberg;
use blake2::Blake2s;
use noir_field::FieldElement;
use sha2::Digest;
use std::collections::BTreeMap;

// Note that the outputs for things like Sha256 need to be computed
// as they may be used in later arithmetic gates

pub struct GadgetCaller;

impl GadgetCaller {
    pub fn solve_gadget_call(
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
        gadget_call: &GadgetCall,
    ) -> Result<(), acir::OPCODE> {
        match gadget_call.name {
            OPCODE::SHA256 => pwg::hash::sha256(initial_witness, gadget_call),
            OPCODE::Blake2s => pwg::hash::blake2s(initial_witness, gadget_call),
            OPCODE::EcdsaSecp256k1 => {
                pwg::signature::ecdsa::secp256k1_prehashed(initial_witness, gadget_call)
            }
            OPCODE::AES => return Err(gadget_call.name),
            OPCODE::MerkleMembership => {
                let mut inputs_iter = gadget_call.inputs.iter();

                let _root = inputs_iter.next().expect("expected a root");
                // let root = input_to_value(initial_witness, _root); // TODO (1a) once we have proper state management, we can use the root that is passed in

                let _leaf = inputs_iter.next().expect("expected a leaf");
                let leaf = input_to_value(initial_witness, _leaf).clone();

                let _index = inputs_iter.next().expect("expected the depth parameter");
                // The value of index should not be set yet; it was created in the evaluator

                let hashpath_indices: Vec<_> = inputs_iter.collect();
                let arity = 2;
                let depth = hashpath_indices.len() / arity;

                let mut merkle_tree = MerkleTree::new(depth as u32);

                let root = merkle_tree.update_leaf(2, leaf); // TODO (1b) We are updating the tree here, because we do not have proper state management yet
                initial_witness.insert(_root.witness, root);
                // XXX: Usually the root would be passed in as a proof, and here we would assert that
                // the root in the tree is equal to the root passed in
                // assert_eq!(root, merkle_tree.root);

                // Update the index

                let index = merkle_tree
                    .find_index_from_leaf(&leaf)
                    .expect("value not found in merkle tree");

                let index_fr = FieldElement::from(index as i128);
                // Set the index here
                initial_witness.insert(_index.witness, index_fr);

                // Update hashpath
                let path = merkle_tree.get_hash_path(index);
                let mut hashpath_indices = hashpath_indices.iter();
                for (left_hash, right_hash) in path.iter().copied() {
                    let left = hashpath_indices.next().unwrap().witness;
                    let right = hashpath_indices.next().unwrap().witness;

                    initial_witness.insert(left, left_hash);
                    initial_witness.insert(right, right_hash);
                }
                let hash_path = flatten_path(path);

                let result = MerkleTree::check_membership(
                    hash_path.iter().collect(),
                    &root,
                    &index_fr,
                    &leaf,
                );

                initial_witness.insert(gadget_call.outputs[0], result);
            }
            OPCODE::SchnorrVerify => {
                // In barretenberg, if the signature fails, then the whole thing fails.
                //
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
                for (i, sig) in signature.iter_mut().enumerate() {
                    let _sig_i = inputs_iter.next().unwrap_or_else(|| {
                        panic!("signature should be 64 bytes long, found only {} bytes", i)
                    });
                    let sig_i = input_to_value(initial_witness, _sig_i);
                    *sig = *sig_i.to_bytes().last().unwrap()
                }

                let mut message = Vec::new();
                for msg in inputs_iter {
                    let msg_i_field = input_to_value(initial_witness, msg);
                    let msg_i = *msg_i_field.to_bytes().last().unwrap();
                    message.push(msg_i);
                }

                let mut barretenberg = Barretenberg::new();

                let result = barretenberg.verify_signature(pub_key, signature, &message);
                if result != FieldElement::one() {
                    dbg!("signature has failed to verify");
                }

                initial_witness.insert(gadget_call.outputs[0], result);
            }
            OPCODE::Pedersen => {
                let inputs_iter = gadget_call.inputs.iter();

                let scalars: Vec<_> = inputs_iter
                    .map(|input| *input_to_value(initial_witness, input))
                    .collect();

                let mut barretenberg = Barretenberg::new();

                let (res_x, res_y) = barretenberg.encrypt(scalars);
                initial_witness.insert(gadget_call.outputs[0], res_x);
                initial_witness.insert(gadget_call.outputs[1], res_y);
            }
            OPCODE::HashToField => {
                // Deal with Blake2s -- XXX: It's not possible for pwg to know that it is Blake2s
                // We need to get this method from the backend
                let mut hasher = Blake2s::new();

                // 0. For each input in the vector of inputs, check if we have their witness assignments (Can do this outside of match, since they all have inputs)
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

                let reduced_res = FieldElement::from_be_bytes_reduce(&result);
                assert_eq!(gadget_call.outputs.len(), 1);

                initial_witness.insert(gadget_call.outputs[0], reduced_res);
            }
            OPCODE::FixedBaseScalarMul => {
                let scalar = initial_witness.get(&gadget_call.inputs[0].witness);
                let scalar = match scalar {
                    None => panic!("cannot find witness assignment for {:?}", scalar),
                    Some(assignment) => assignment,
                };
                let mut barretenberg = Barretenberg::new();
                let (pub_x, pub_y) = barretenberg.fixed_base(scalar);

                initial_witness.insert(gadget_call.outputs[0], pub_x);
                initial_witness.insert(gadget_call.outputs[1], pub_y);
            }
        }
        Ok(())
    }
}
