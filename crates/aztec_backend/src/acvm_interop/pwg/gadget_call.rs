use super::merkle::{flatten_path, MerkleTree};
use crate::barretenberg_rs::Barretenberg;
use acvm::acir::{circuit::gate::GadgetCall, native_types::Witness, OPCODE};
use acvm::pwg::{self, input_to_value};
use acvm::FieldElement;
use blake2::Blake2s;
use sha2::Digest;
use std::collections::BTreeMap;

// Note that the outputs for things like Sha256 need to be computed
// as they may be used in later arithmetic gates

pub struct GadgetCaller;

impl GadgetCaller {
    pub fn solve_gadget_call(
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
        gadget_call: &GadgetCall,
    ) -> Result<(), OPCODE> {
        match gadget_call.name {
            OPCODE::SHA256 => pwg::hash::sha256(initial_witness, gadget_call),
            OPCODE::Blake2s => pwg::hash::blake2s(initial_witness, gadget_call),
            OPCODE::EcdsaSecp256k1 => {
                pwg::signature::ecdsa::secp256k1_prehashed(initial_witness, gadget_call)
            }
            OPCODE::AES => return Err(gadget_call.name),
            OPCODE::MerkleMembership => {
                // let mut inputs_iter = gadget_call.inputs.iter();

                // let _root = inputs_iter.next().expect("expected a root");
                // let root = *input_to_value(initial_witness, _root);
            
                // let _leaf = inputs_iter.next().expect("expected a leaf");
                // let leaf = *input_to_value(initial_witness, _leaf);
            
                // let _index = inputs_iter.next().expect("expected the depth parameter");
                // let index = *input_to_value(initial_witness, _index);

                const SHOULD_INSERT: bool = false;

                let merkle_data =
                    process_merkle_gadget(initial_witness, gadget_call, SHOULD_INSERT);
                assert!(merkle_data.new_root.is_none());

                // let hash_path = flatten_path(merkle_data.hashpath);
                let result = MerkleTree::check_membership(
                    merkle_data.hashpath.iter().collect(),
                    &merkle_data.old_root,
                    &merkle_data.index,
                    &merkle_data.leaf,
                );

                initial_witness.insert(gadget_call.outputs[0], result);
            }
            OPCODE::InsertRegularMerkle => {
                const SHOULD_INSERT: bool = true;

                let merkle_data =
                    process_merkle_gadget(initial_witness, gadget_call, SHOULD_INSERT);

                let new_root =
                    merkle_data.new_root.expect("new root should be computed for insertions");

                initial_witness.insert(gadget_call.outputs[0], new_root);
            }
            OPCODE::SchnorrVerify => {
                // In barretenberg, if the signature fails, then the whole thing fails.
                //
                use std::convert::TryInto;

                let mut inputs_iter = gadget_call.inputs.iter();

                let _pub_key_x = inputs_iter.next().expect("expected `x` component for public key");
                let pub_key_x = input_to_value(initial_witness, _pub_key_x).to_bytes();

                let _pub_key_y = inputs_iter.next().expect("expected `y` component for public key");
                let pub_key_y = input_to_value(initial_witness, _pub_key_y).to_bytes();

                let pub_key_bytes: Vec<u8> =
                    pub_key_x.iter().copied().chain(pub_key_y.to_vec()).collect();
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

                let scalars: Vec<_> =
                    inputs_iter.map(|input| *input_to_value(initial_witness, input)).collect();

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
            OPCODE::ToBits => unreachable!(),
        }
        Ok(())
    }
}

struct MerkleData {
    hashpath: Vec<FieldElement>,
    old_root: FieldElement,
    new_root: Option<FieldElement>,
    leaf: FieldElement,
    index: FieldElement,
}
fn process_merkle_gadget(
    initial_witness: &mut BTreeMap<Witness, FieldElement>,
    gadget_call: &GadgetCall,
    should_insert: bool,
) -> MerkleData {
    let mut inputs_iter = gadget_call.inputs.iter();

    let _root = inputs_iter.next().expect("expected a root");
    let root = *input_to_value(initial_witness, _root);

    let _leaf = inputs_iter.next().expect("expected a leaf");
    let leaf = *input_to_value(initial_witness, _leaf);

    let _index = inputs_iter.next().expect("expected the depth parameter");
    let index = *input_to_value(initial_witness, _index);

    let path: Vec<_> = inputs_iter
        .map(|input| *input_to_value(initial_witness, input))
        .collect();

    let new_root = if should_insert {
        // To insert, we first fetch the index of the fist empty leaf
        // Then we insert into the trie to compute the new root
        // It should be inserted into that same empty spot
        // let _index = merkle_tree.find_index_for_empty_leaf();
        // let new_root = merkle_tree.update_leaf(_index, leaf);
        
        // To insert we first check that the index we want to insert into of the current merkle tree is empty
        assert!(MerkleTree::check_membership(
            path.iter().collect(), 
            &root, 
            &index, 
            &FieldElement::zero()).is_one() 
        );

        // TODO: generate new root
        let (new_hash_path, new_root) = MerkleTree::insert_leaf(
            path.iter().collect(),
            &index,
            &leaf
        );

        assert!(MerkleTree::check_membership(
            new_hash_path.iter().collect(),
            &new_root,
            &index,
            &leaf).is_one()
        );

        Some(new_root)
    } else {
        None
    };

    MerkleData { hashpath: path, old_root: root, new_root, leaf, index}
}
