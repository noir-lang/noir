use crate::address::{AztecAddress, EthAddress};
use crate::mocked::VerificationKey;
use crate::abis::function_selector::FunctionSelector;
use crate::abis::function_leaf_preimage::FunctionLeafPreimage;
use crate::abis::new_contract_data::NewContractData as ContractLeafPreimage;
use crate::abis::function_data::FunctionData;
use crate::abis::side_effect::{SideEffect};
use crate::utils::uint256::U256;
use crate::utils::bounded_vec::BoundedVec;
use crate::constants::{
        ARGS_HASH_CHUNK_COUNT,
        ARGS_HASH_CHUNK_LENGTH,
        CONTRACT_TREE_HEIGHT, 
        FUNCTION_TREE_HEIGHT, 
        NOTE_HASH_TREE_HEIGHT,
        NUM_FIELDS_PER_SHA256,
        GENERATOR_INDEX__SILOED_COMMITMENT,
        GENERATOR_INDEX__OUTER_NULLIFIER,
        GENERATOR_INDEX__VK,
        GENERATOR_INDEX__CONSTRUCTOR,
        GENERATOR_INDEX__PARTIAL_ADDRESS,
        GENERATOR_INDEX__CONTRACT_ADDRESS,
        GENERATOR_INDEX__COMMITMENT_NONCE,
        GENERATOR_INDEX__UNIQUE_COMMITMENT,
        GENERATOR_INDEX__FUNCTION_ARGS,
};

use dep::std::hash::{pedersen_hash_with_separator, sha256};

pub fn sha256_to_field<N>(bytes_to_hash: [u8; N]) -> Field {
    let sha256_hashed = sha256(bytes_to_hash);

    // Convert it to a field element
    let mut v = 1;
    let mut high = 0 as Field;
    let mut low = 0 as Field;

    for i in 0..16 {
        high = high + (sha256_hashed[15 - i] as Field) * v;
        low = low + (sha256_hashed[16 + 15 - i] as Field) * v;
        v = v * 256;
    }

    // Abuse that a % p + b % p = (a + b) % p and that low < p
    let hash_in_a_field = low + high * v;

    hash_in_a_field
}

pub fn hash_args<N>(args: [Field; N]) -> Field {
    if args.len() == 0 {
        0
    } else {
        let mut chunks_hashes = [0; ARGS_HASH_CHUNK_COUNT];
        for i in 0..ARGS_HASH_CHUNK_COUNT {
            let mut chunk_hash = 0;
            let start_chunk_index = i * ARGS_HASH_CHUNK_LENGTH;
            if start_chunk_index < (args.len() as u32) {
                let mut chunk_args = [0; ARGS_HASH_CHUNK_LENGTH];
                for j in 0..ARGS_HASH_CHUNK_LENGTH {
                    let item_index = i * ARGS_HASH_CHUNK_LENGTH + j;
                    if item_index < (args.len() as u32) {
                        chunk_args[j] = args[item_index];
                    }
                }
                chunk_hash = pedersen_hash(chunk_args, GENERATOR_INDEX__FUNCTION_ARGS);
            }
            chunks_hashes[i] = chunk_hash;
        }
        pedersen_hash(chunks_hashes, GENERATOR_INDEX__FUNCTION_ARGS)
    }
}

// Checks that `value` is a member of a merkle tree with root `root` at position `index`
// The witness being the `sibling_path`
pub fn assert_check_membership<N>(value: Field, index: Field, sibling_path: [Field; N], root: Field) {
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
pub fn root_from_sibling_path<N>(leaf: Field, leaf_index: Field, sibling_path: [Field; N]) -> Field {
    let mut node = leaf;
    let indices = leaf_index.to_le_bits(N);

    for i in 0..N {
        let (hash_left, hash_right) = if indices[i] == 1 {
            (sibling_path[i], node)
        } else {
            (node, sibling_path[i])
        };
        node = merkle_hash(hash_left, hash_right);
    }
    node
}

// Calculate the function tree root from the sibling path and leaf preimage.
//
// TODO: The cpp code passes in components of the FunctionLeafPreimage and then 
// builds it up. We should build it up and then pass the leaf preimage as a parameter.
// We can then choose to have a general method that takes in anything hashable
// and deduplicate the logic in `contract_tree_root_from_siblings`
pub fn function_tree_root_from_siblings(
    selector: FunctionSelector,
    is_internal: bool,
    is_private: bool,
    vk_hash: Field,
    acir_hash: Field,
    function_leaf_index: Field,
    function_leaf_sibling_path: [Field; FUNCTION_TREE_HEIGHT]
) -> Field {
    let function_leaf_preimage = FunctionLeafPreimage { selector, is_internal, is_private, vk_hash, acir_hash };

    let function_leaf = function_leaf_preimage.hash();

    let function_tree_root = root_from_sibling_path(function_leaf, function_leaf_index, function_leaf_sibling_path);

    function_tree_root
}

// Calculate the contract tree root from the sibling path and leaf preimage.
pub fn contract_tree_root_from_siblings(
    function_tree_root: Field,
    storage_contract_address: AztecAddress,
    portal_contract_address: EthAddress,
    contract_leaf_index: Field,
    contract_leaf_sibling_path: [Field; CONTRACT_TREE_HEIGHT]
) -> Field {
    //TODO(Kev): if we use shorthand syntax here, we get an error as expected,
    // since variable name is `storage_contract_address` but the span is incorrect.
    let contract_leaf_preimage = ContractLeafPreimage { contract_address: storage_contract_address, portal_contract_address, function_tree_root };

    let contract_leaf = contract_leaf_preimage.hash();

    let computed_contract_tree_root = root_from_sibling_path(contract_leaf, contract_leaf_index, contract_leaf_sibling_path);

    computed_contract_tree_root
}

pub fn read_request_root_from_siblings(
    read_request: Field,
    leaf_index: Field,
    sibling_path: [Field; NOTE_HASH_TREE_HEIGHT]
) -> Field {
    root_from_sibling_path(read_request, leaf_index, sibling_path)
}

pub fn silo_commitment(address: AztecAddress, inner_commitment: Field) -> Field {
    pedersen_hash(
        [
        address.to_field(),
        inner_commitment
    ],
        GENERATOR_INDEX__SILOED_COMMITMENT
    )
}

pub fn silo_nullifier(address: AztecAddress, nullifier: Field) -> Field {
    pedersen_hash(
        [
        address.to_field(),
        nullifier
    ],
        GENERATOR_INDEX__OUTER_NULLIFIER
    )
}

fn merkle_hash(left: Field, right: Field) -> Field {
    pedersen_hash([left, right], 0)
}

pub fn stdlib_recursion_verification_key_compress_native_vk(_vk: VerificationKey) -> Field {
    // Original cpp code
    // stdlib::recursion::verification_key<CT::bn254>::compress_native(private_call.vk, GeneratorIndex::VK);
    // The above cpp method is only ever called on verification key, so it has been special cased here
    let _hash_index = GENERATOR_INDEX__VK;
    0
}

// TODO CPP uses blake2s for this
pub fn compute_new_contract_address_hash(new_contract_address: AztecAddress) -> Field {
    dep::std::hash::pedersen_hash([new_contract_address.to_field()])
}

pub fn compute_l2_to_l1_hash(
    contract_address: AztecAddress,
    rollup_version_id: Field,
    portal_contract_address: EthAddress,
    chain_id: Field,
    content: Field
) -> Field {
    let mut bytes: BoundedVec<u8, 160> = BoundedVec::new(0);

    let inputs = [
        contract_address.to_field(), rollup_version_id, portal_contract_address.to_field(), chain_id, content
    ];
    for i in 0..inputs.len() {
        // TODO are bytes be in fr.to_buffer() ?
        let item_bytes = inputs[i].to_be_bytes(32);
        for j in 0..32 {
            bytes.push(item_bytes[j]);
        }
    }

    sha256_to_field(bytes.storage)
}

pub fn compute_constructor_hash(
    function_data: FunctionData,
    args_hash: Field,
    constructor_vk_hash: Field
) -> Field {
    let function_data_hash = function_data.hash();

    pedersen_hash(
        [
        function_data_hash,
        args_hash,
        constructor_vk_hash
    ],
        GENERATOR_INDEX__CONSTRUCTOR
    )
}

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
pub fn accumulate_sha256(input: [U128; 4]) -> [Field; NUM_FIELDS_PER_SHA256] {
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
    let mut hash_input_flattened = [0; 64];
    for offset in 0..4 {
        let input_as_bytes = input[offset].to_be_bytes();
        for byte_index in 0..16 {
            hash_input_flattened[offset * 16 + byte_index] = input_as_bytes[byte_index];
        }
    }

    let sha_digest = dep::std::hash::sha256(hash_input_flattened);

    U256::from_bytes32(sha_digest).to_u128_limbs()
}

pub fn compute_logs_hash(
    previous_log_hash: [Field; 2],
    current_log_hash: [Field; 2]
) -> [Field; NUM_FIELDS_PER_SHA256] {
    accumulate_sha256(
        [
        U128::from_integer(previous_log_hash[0]),
        U128::from_integer(previous_log_hash[1]),
        U128::from_integer(current_log_hash[0]),
        U128::from_integer(current_log_hash[1])
    ]
    )
}

pub fn compute_commitment_nonce(first_nullifier: Field, commitment_index: Field) -> Field {
    pedersen_hash(
        [
        first_nullifier,
        commitment_index
    ],
        GENERATOR_INDEX__COMMITMENT_NONCE
    )
}

pub fn compute_unique_siloed_commitment(nonce: Field, siloed_commitment: Field) -> Field {
    pedersen_hash(
        [
        nonce,
        siloed_commitment
    ],
        GENERATOR_INDEX__UNIQUE_COMMITMENT
    )
}

pub fn compute_unique_siloed_commitments<N>(
    first_nullifier: Field,
    siloed_commitments: [SideEffect; N]
) -> [SideEffect; N] {
    let mut unique_siloed_commitments = [SideEffect::empty(); N];
    for i in 0..N {
        let siloed_commitment = siloed_commitments[i];
        if siloed_commitment.value != 0 {
            let nonce = compute_commitment_nonce(first_nullifier, i);
            unique_siloed_commitments[i] = SideEffect {
                value: compute_unique_siloed_commitment(nonce, siloed_commitment.value),
                counter: siloed_commitment.counter
                };
        }
    }
    unique_siloed_commitments
}

pub fn pedersen_hash<N>(inputs: [Field; N], hash_index: u32) -> Field {
    dep::std::hash::pedersen_hash_with_separator(inputs, hash_index)
}
