#pragma once
#include "msgpack/v3/adaptor/detail/cpp11_define_map_decl.hpp"

#include "aztec3/constants.hpp"

#include "barretenberg/serialize/msgpack_impl/name_value_pair_macro.hpp"

namespace aztec3::circuits::abis {

// Represents constants during serialization (only)
struct ConstantsPacker {
    template <typename Packer> void msgpack_pack(Packer& packer) const
    {
        auto pack = [&](auto&... args) {
            msgpack::type::define_map<decltype(args)...>{ args... }.msgpack_pack(packer);
        };

        // Note: NVP macro can handle up to 20 arguments so we call it multiple times here. If adding a new constant
        // add it to the last call or introduce a new one if the last call is already "full".
        pack(NVP(ARGS_LENGTH,
                 RETURN_VALUES_LENGTH,
                 MAX_NEW_COMMITMENTS_PER_CALL,
                 MAX_NEW_NULLIFIERS_PER_CALL,
                 MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL,
                 MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
                 MAX_NEW_L2_TO_L1_MSGS_PER_CALL,
                 MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL,
                 MAX_PUBLIC_DATA_READS_PER_CALL,
                 MAX_READ_REQUESTS_PER_CALL,
                 MAX_NEW_COMMITMENTS_PER_TX,
                 MAX_NEW_NULLIFIERS_PER_TX,
                 MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX,
                 MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX,
                 MAX_NEW_L2_TO_L1_MSGS_PER_TX,
                 MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
                 MAX_PUBLIC_DATA_READS_PER_TX,
                 MAX_NEW_CONTRACTS_PER_TX,
                 MAX_OPTIONALLY_REVEALED_DATA_LENGTH_PER_TX,
                 MAX_READ_REQUESTS_PER_TX),
             NVP(NUM_ENCRYPTED_LOGS_HASHES_PER_TX,
                 NUM_UNENCRYPTED_LOGS_HASHES_PER_TX,
                 NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP,
                 KERNELS_PER_BASE_ROLLUP,
                 VK_TREE_HEIGHT,
                 FUNCTION_TREE_HEIGHT,
                 CONTRACT_TREE_HEIGHT,
                 PRIVATE_DATA_TREE_HEIGHT,
                 PUBLIC_DATA_TREE_HEIGHT,
                 NULLIFIER_TREE_HEIGHT,
                 L1_TO_L2_MSG_TREE_HEIGHT,
                 ROLLUP_VK_TREE_HEIGHT,
                 CONTRACT_SUBTREE_HEIGHT,
                 CONTRACT_SUBTREE_SIBLING_PATH_LENGTH,
                 PRIVATE_DATA_SUBTREE_HEIGHT,
                 PRIVATE_DATA_SUBTREE_SIBLING_PATH_LENGTH,
                 NULLIFIER_SUBTREE_HEIGHT,
                 HISTORIC_BLOCKS_TREE_HEIGHT,
                 NULLIFIER_SUBTREE_SIBLING_PATH_LENGTH,
                 L1_TO_L2_MSG_SUBTREE_HEIGHT),
             NVP(L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH,
                 FUNCTION_SELECTOR_NUM_BYTES,
                 MAPPING_SLOT_PEDERSEN_SEPARATOR,
                 NUM_FIELDS_PER_SHA256,
                 L1_TO_L2_MESSAGE_LENGTH,
                 L1_TO_L2_MESSAGE_ORACLE_CALL_LENGTH,
                 MAX_NOTE_FIELDS_LENGTH,
                 GET_NOTE_ORACLE_RETURN_LENGTH,
                 MAX_NOTES_PER_PAGE,
                 VIEW_NOTE_ORACLE_RETURN_LENGTH,
                 CALL_CONTEXT_LENGTH,
                 CONSTANT_HISTORIC_BLOCK_DATA_LENGTH,
                 FUNCTION_DATA_LENGTH,
                 CONTRACT_DEPLOYMENT_DATA_LENGTH,
                 PRIVATE_CIRCUIT_PUBLIC_INPUTS_LENGTH,
                 CONTRACT_STORAGE_UPDATE_REQUEST_LENGTH,
                 CONTRACT_STORAGE_READ_LENGTH,
                 PUBLIC_CIRCUIT_PUBLIC_INPUTS_LENGTH,
                 GET_NOTES_ORACLE_RETURN_LENGTH,
                 EMPTY_NULLIFIED_COMMITMENT),
             NVP(CALL_PRIVATE_FUNCTION_RETURN_SIZE,
                 PUBLIC_CIRCUIT_PUBLIC_INPUTS_HASH_INPUT_LENGTH,
                 PRIVATE_CIRCUIT_PUBLIC_INPUTS_HASH_INPUT_LENGTH,
                 KERNELS_PER_BASE_ROLLUP,
                 COMMITMENTS_NUM_BYTES_PER_BASE_ROLLUP,
                 NULLIFIERS_NUM_BYTES_PER_BASE_ROLLUP,
                 PUBLIC_DATA_WRITES_NUM_BYTES_PER_BASE_ROLLUP,
                 CONTRACTS_NUM_BYTES_PER_BASE_ROLLUP,
                 CONTRACT_DATA_NUM_BYTES_PER_BASE_ROLLUP,
                 CONTRACT_DATA_NUM_BYTES_PER_BASE_ROLLUP_UNPADDED,
                 L2_TO_L1_MSGS_NUM_BYTES_PER_BASE_ROLLUP,
                 LOGS_HASHES_NUM_BYTES_PER_BASE_ROLLUP));  // <-- Add names of new constants here
    }
};

struct GeneratorIndexPacker {
    template <typename Packer> void msgpack_pack(Packer& packer) const
    {
        auto pack = [&](auto&... args) {
            msgpack::type::define_map<decltype(args)...>{ args... }.msgpack_pack(packer);
        };

        int COMMITMENT = GeneratorIndex::COMMITMENT;
        int COMMITMENT_NONCE = GeneratorIndex::COMMITMENT_NONCE;
        int UNIQUE_COMMITMENT = GeneratorIndex::UNIQUE_COMMITMENT;
        int SILOED_COMMITMENT = GeneratorIndex::SILOED_COMMITMENT;
        int NULLIFIER = GeneratorIndex::NULLIFIER;
        int INITIALISATION_NULLIFIER = GeneratorIndex::INITIALISATION_NULLIFIER;
        int OUTER_NULLIFIER = GeneratorIndex::OUTER_NULLIFIER;
        int PUBLIC_DATA_READ = GeneratorIndex::PUBLIC_DATA_READ;
        int PUBLIC_DATA_UPDATE_REQUEST = GeneratorIndex::PUBLIC_DATA_UPDATE_REQUEST;
        int FUNCTION_DATA = GeneratorIndex::FUNCTION_DATA;
        int FUNCTION_LEAF = GeneratorIndex::FUNCTION_LEAF;
        int CONTRACT_DEPLOYMENT_DATA = GeneratorIndex::CONTRACT_DEPLOYMENT_DATA;
        int CONSTRUCTOR = GeneratorIndex::CONSTRUCTOR;
        int CONSTRUCTOR_ARGS = GeneratorIndex::CONSTRUCTOR_ARGS;
        int CONTRACT_ADDRESS = GeneratorIndex::CONTRACT_ADDRESS;
        int CONTRACT_LEAF = GeneratorIndex::CONTRACT_LEAF;
        int CALL_CONTEXT = GeneratorIndex::CALL_CONTEXT;
        int CALL_STACK_ITEM = GeneratorIndex::CALL_STACK_ITEM;
        int CALL_STACK_ITEM_2 = GeneratorIndex::CALL_STACK_ITEM_2;
        int L1_TO_L2_MESSAGE_SECRET = GeneratorIndex::L1_TO_L2_MESSAGE_SECRET;
        int L2_TO_L1_MSG = GeneratorIndex::L2_TO_L1_MSG;
        int TX_CONTEXT = GeneratorIndex::TX_CONTEXT;
        int PUBLIC_LEAF_INDEX = GeneratorIndex::PUBLIC_LEAF_INDEX;
        int PUBLIC_DATA_LEAF = GeneratorIndex::PUBLIC_DATA_LEAF;
        int SIGNED_TX_REQUEST = GeneratorIndex::SIGNED_TX_REQUEST;
        int GLOBAL_VARIABLES = GeneratorIndex::GLOBAL_VARIABLES;
        int PARTIAL_ADDRESS = GeneratorIndex::PARTIAL_ADDRESS;
        int TX_REQUEST = GeneratorIndex::TX_REQUEST;
        int SIGNATURE_PAYLOAD = GeneratorIndex::SIGNATURE_PAYLOAD;
        int VK = GeneratorIndex::VK;
        int PRIVATE_CIRCUIT_PUBLIC_INPUTS = GeneratorIndex::PRIVATE_CIRCUIT_PUBLIC_INPUTS;
        int PUBLIC_CIRCUIT_PUBLIC_INPUTS = GeneratorIndex::PUBLIC_CIRCUIT_PUBLIC_INPUTS;
        int FUNCTION_ARGS = GeneratorIndex::FUNCTION_ARGS;


        // Note: NVP macro can handle up to 20 arguments so we call it multiple times here. If adding a new constant
        // add it to the last call or introduce a new one if the last call is already "full".
        pack(NVP(COMMITMENT,
                 COMMITMENT_NONCE,
                 UNIQUE_COMMITMENT,
                 SILOED_COMMITMENT,
                 NULLIFIER,
                 INITIALISATION_NULLIFIER,
                 OUTER_NULLIFIER,
                 PUBLIC_DATA_READ,
                 PUBLIC_DATA_UPDATE_REQUEST,
                 FUNCTION_DATA,
                 FUNCTION_LEAF,
                 CONTRACT_DEPLOYMENT_DATA,
                 CONSTRUCTOR,
                 CONSTRUCTOR_ARGS,
                 CONTRACT_ADDRESS,
                 CONTRACT_LEAF,
                 CALL_CONTEXT,
                 CALL_STACK_ITEM,
                 CALL_STACK_ITEM_2,
                 L1_TO_L2_MESSAGE_SECRET),
             NVP(L2_TO_L1_MSG,
                 TX_CONTEXT,
                 PUBLIC_LEAF_INDEX,
                 PUBLIC_DATA_LEAF,
                 SIGNED_TX_REQUEST,
                 GLOBAL_VARIABLES,
                 PARTIAL_ADDRESS,
                 TX_REQUEST,
                 SIGNATURE_PAYLOAD,
                 VK,
                 PRIVATE_CIRCUIT_PUBLIC_INPUTS,
                 PUBLIC_CIRCUIT_PUBLIC_INPUTS,
                 FUNCTION_ARGS));
    }
};

struct StorageSlotGeneratorIndexPacker {
    template <typename Packer> void msgpack_pack(Packer& packer) const
    {
        auto pack = [&](auto&... args) {
            msgpack::type::define_map<decltype(args)...>{ args... }.msgpack_pack(packer);
        };

        int BASE_SLOT = StorageSlotGeneratorIndex::BASE_SLOT;
        int MAPPING_SLOT = StorageSlotGeneratorIndex::MAPPING_SLOT;
        int MAPPING_SLOT_PLACEHOLDER = StorageSlotGeneratorIndex::MAPPING_SLOT_PLACEHOLDER;

        pack(NVP(BASE_SLOT, MAPPING_SLOT, MAPPING_SLOT_PLACEHOLDER));
    }
};

struct PrivateStateNoteGeneratorIndexPacker {
    template <typename Packer> void msgpack_pack(Packer& packer) const
    {
        auto pack = [&](auto&... args) {
            msgpack::type::define_map<decltype(args)...>{ args... }.msgpack_pack(packer);
        };

        int VALUE = PrivateStateNoteGeneratorIndex::VALUE;
        int OWNER = PrivateStateNoteGeneratorIndex::OWNER;
        int CREATOR = PrivateStateNoteGeneratorIndex::CREATOR;
        int SALT = PrivateStateNoteGeneratorIndex::SALT;
        int NONCE = PrivateStateNoteGeneratorIndex::NONCE;
        int MEMO = PrivateStateNoteGeneratorIndex::MEMO;
        int IS_DUMMY = PrivateStateNoteGeneratorIndex::IS_DUMMY;

        pack(NVP(VALUE, OWNER, CREATOR, SALT, NONCE, MEMO, IS_DUMMY));
    }
};

struct PrivateStateTypePacker {
    template <typename Packer> void msgpack_pack(Packer& packer) const
    {
        auto pack = [&](auto&... args) {
            msgpack::type::define_map<decltype(args)...>{ args... }.msgpack_pack(packer);
        };

        int PARTITIONED = PrivateStateType::PARTITIONED;
        int WHOLE = PrivateStateType::WHOLE;

        pack(NVP(PARTITIONED, WHOLE));
    }
};

}  // namespace aztec3::circuits::abis