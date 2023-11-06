use crate::address::{Address, EthAddress};
use crate::mocked::VerificationKey;
use crate::point::Point;
use crate::abis::function_selector::FunctionSelector;
use crate::abis::function_leaf_preimage::FunctionLeafPreimage;
use crate::abis::new_contract_data::NewContractData as ContractLeafPreimage;
use crate::abis::function_data::FunctionData;
use crate::utils::uint128::U128;
use crate::utils::uint256::U256;
use crate::utils::bounded_vec::BoundedVec;

use dep::aztec::{
    constants_gen,
    constants_gen::{CONTRACT_TREE_HEIGHT, FUNCTION_TREE_HEIGHT, NOTE_HASH_TREE_HEIGHT},
    hash::sha256_to_field,
};

// Checks that `value` is a member of a merkle tree with root `root` at position `index`
// The witness being the `sibling_path`
pub fn assert_check_membership<N>(value : Field, index : Field, sibling_path : [Field; N], root : Field) {
    let calculated_root = root_from_sibling_path(value, index, sibling_path);
    assert(calculated_root == root, "membership check failed");
}

// Calculate the Merkle tree root from the sibling path and leaf.
//
// The leaf is hashed with its sibling, and then the result is hashed
// with the next sibling etc in the path. The last hash is the root.
//
// TODO(David/Someone): The cpp code is using a uint256, whereas its
// TODO a bit simpler in Noir to just have a bit array.
// TODO: I'd generally like to avoid u256 for algorithms like 
// this because it means we never even need to consider cases where 
// the index is greater than p.
pub fn root_from_sibling_path<N>(leaf : Field, leaf_index : Field, sibling_path : [Field; N]) -> Field {
    let mut node = leaf;
    let indices = leaf_index.to_le_bits(N);

    for i in 0..N {
        // indices[i]; // This line will fail.
        if indices[i] == 1 {
            node = merkle_hash(sibling_path[i], node);
        } else {
            node = merkle_hash(node, sibling_path[i]);
        }
    }
    node
}

// Calculate the function tree root from the sibling path and leaf preimage.
//
// TODO: The cpp code passes in components of the FunctionLeafPreimage and then 
// builds it up. We should build it up and then pass the leaf preimage as a parameter.
// We can then choose to have a general method that takes in anything hashable
// and deduplicate the logic in `contract_tree_root_from_siblings`
pub fn function_tree_root_from_siblings(selector : FunctionSelector, is_internal : bool, is_private : bool, vk_hash : Field, acir_hash : Field, function_leaf_index : Field, function_leaf_sibling_path : [Field; FUNCTION_TREE_HEIGHT]) -> Field {
    let function_leaf_preimage = FunctionLeafPreimage{
        selector: selector,
        is_internal : is_internal,
        is_private : is_private,
        vk_hash : vk_hash,
        acir_hash : acir_hash,
    };

    let function_leaf = function_leaf_preimage.hash();

    let function_tree_root =
        root_from_sibling_path(function_leaf, function_leaf_index, function_leaf_sibling_path);

    function_tree_root
}

// Calculate the contract tree root from the sibling path and leaf preimage.
pub fn contract_tree_root_from_siblings(function_tree_root : Field, storage_contract_address : Address, portal_contract_address : EthAddress, contract_leaf_index : Field,contract_leaf_sibling_path : [Field; CONTRACT_TREE_HEIGHT]) -> Field {
    //TODO(Kev): if we use shorthand syntax here, we get an error as expected,
    // since variable name is `storage_contract_address` but the span is incorrect.
    let contract_leaf_preimage = ContractLeafPreimage { contract_address: storage_contract_address,
                                    portal_contract_address,
                                    function_tree_root };
        
    let contract_leaf = contract_leaf_preimage.hash();

    let computed_contract_tree_root =
        root_from_sibling_path(contract_leaf, contract_leaf_index, contract_leaf_sibling_path);
    
    computed_contract_tree_root
}

pub fn read_request_root_from_siblings(read_request : Field, leaf_index : Field, sibling_path : [Field; NOTE_HASH_TREE_HEIGHT]) -> Field {
    root_from_sibling_path(read_request, leaf_index, sibling_path)
}

pub fn silo_commitment(address : Address, inner_commitment : Field) -> Field {
    dep::std::hash::pedersen_hash_with_separator([
        address.to_field(),
        inner_commitment,
    ], constants_gen::GENERATOR_INDEX__SILOED_COMMITMENT)
}

pub fn silo_nullifier(address : Address, nullifier : Field) -> Field {
    dep::std::hash::pedersen_hash_with_separator([
        address.to_field(),
        nullifier,
    ], constants_gen::GENERATOR_INDEX__OUTER_NULLIFIER)
}

fn merkle_hash(left : Field, right : Field) -> Field {
    dep::std::hash::pedersen_hash_with_separator([left, right], 0)
}

pub fn stdlib_recursion_verification_key_compress_native_vk(_vk : VerificationKey) -> Field {
    // Original cpp code
    // stdlib::recursion::verification_key<CT::bn254>::compress_native(private_call.vk, GeneratorIndex::VK);
    // The above cpp method is only ever called on verification key, so it has been special cased here
    let _hash_index = constants_gen::GENERATOR_INDEX__VK;
    0
}

// TODO CPP uses blake2s for this
pub fn compute_new_contract_address_hash(new_contract_address : Address) -> Field {
    dep::std::hash::pedersen_hash([new_contract_address.to_field()])
}

pub fn compute_l2_to_l1_hash(contract_address : Address, rollup_version_id: Field, portal_contract_address : EthAddress, chain_id : Field, content : Field) -> Field {
    let mut bytes: BoundedVec<u8, 160> = BoundedVec::new(0);

    let inputs = [contract_address.to_field(), rollup_version_id, portal_contract_address.to_field(), chain_id, content];
    for i in 0..inputs.len() {
        // TODO are bytes be in fr.to_buffer() ?
        let item_bytes = inputs[i].to_be_bytes(32);
        for j in 0..32 {
            bytes.push(item_bytes[j]);
        }
    }

    sha256_to_field(bytes.storage)
}

pub fn compute_constructor_hash(function_data : FunctionData, args_hash : Field, constructor_vk_hash : Field) -> Field {
    let function_data_hash = function_data.hash();

    dep::std::hash::pedersen_hash_with_separator([
        function_data_hash,
        args_hash,
        constructor_vk_hash
    ], constants_gen::GENERATOR_INDEX__CONSTRUCTOR)
}

// sha256 hash is stored in two fields to accommodate all 256-bits of the hash
global NUM_FIELDS_PER_SHA256 : Field = 2;

// Computes sha256 hash of 2 input hashes stored in 4 fields.
// 
// This method is bn254 specific. Two fields is needed in order to 
// encode the sha256 output. It can be abstracted away with any 4-2 hash function.
//
// TODO(Jan and David): This is used for the encrypted_log hashes.
// Can we check to see if we can just use hash_to_field or pedersen_compress here?
//
// Returning a Field would be desirable because then this can be replaced with 
// poseidon without changing the rest of the code
//
fn accumulate_sha256(input : [U128; 4]) -> [Field; NUM_FIELDS_PER_SHA256] {
    // This is a note about the cpp code, since it takes an array of Fields
    // instead of a U128.
    // 4 Field elements when converted to bytes will usually 
    // occupy 4 * 32 = 128 bytes.
    // However, this function is making the assumption that each Field 
    // only occupies 128 bits.
    //
    // TODO(David): This does not seem to be getting guaranteed anywhere in the code?
    //
    // Concatenate 4 u128 bit integers into a byte array.
    let mut hash_input_flattened = [0;64];
    for offset in 0..4 {
        let input_as_bytes = input[offset].to_be_bytes();
        for byte_index in 0..16 {
            hash_input_flattened[offset * 16 + byte_index] = input_as_bytes[byte_index];
        }
    }

    let sha_digest = dep::std::hash::sha256(hash_input_flattened);

    U256::from_bytes32(sha_digest).to_u128_limbs()
}

pub fn compute_logs_hash(previous_log_hash : [Field;2], current_log_hash : [Field;2]) -> [Field; NUM_FIELDS_PER_SHA256] {
    accumulate_sha256([
        U128::from_field(previous_log_hash[0]),
        U128::from_field(previous_log_hash[1]),
        U128::from_field(current_log_hash[0]),
        U128::from_field(current_log_hash[1])
    ])
}

pub fn compute_partial_address(contract_address_salt : Field, function_tree_root : Field, constructor_hash : Field) -> Field {
    dep::std::hash::pedersen_hash_with_separator([
        // TODO why the zeroes?
        0,
        0,
        contract_address_salt,
        function_tree_root,
        constructor_hash
    ],constants_gen::GENERATOR_INDEX__PARTIAL_ADDRESS)
}

pub fn compute_contract_address_from_partial(point : Point, partial_address : Field) -> Address {
    let field = dep::std::hash::pedersen_hash_with_separator([
        point.x,
        point.y,
        partial_address
    ],constants_gen::GENERATOR_INDEX__CONTRACT_ADDRESS);
    Address::from_field(field)
}

pub fn compute_commitment_nonce(first_nullifier : Field, commitment_index : Field) -> Field {
    dep::std::hash::pedersen_hash_with_separator([
        first_nullifier,
        commitment_index
    ], constants_gen::GENERATOR_INDEX__COMMITMENT_NONCE)
}

pub fn compute_unique_siloed_commitment(nonce: Field, siloed_commitment: Field) -> Field {
    dep::std::hash::pedersen_hash_with_separator([
        nonce,
        siloed_commitment
    ], constants_gen::GENERATOR_INDEX__UNIQUE_COMMITMENT)
}

pub fn compute_unique_siloed_commitments<N>(first_nullifier: Field, siloed_commitments: [Field; N]) -> [Field; N] {
    let mut unique_siloed_commitments = [0; N];
    for i in 0..N {
        let siloed_commitment = siloed_commitments[i];
        if siloed_commitment != 0 {
            let nonce = compute_commitment_nonce(first_nullifier, i);
            unique_siloed_commitments[i] = compute_unique_siloed_commitment(nonce, siloed_commitment);
        }
    }
    unique_siloed_commitments
}