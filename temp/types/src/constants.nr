use crate::{address::AztecAddress, traits::FromField};

pub global MAX_FIELD_VALUE: Field =
    21888242871839275222246405745257275088548364400416034343698204186575808495616;
pub global ARGS_LENGTH: u32 = 16;
// WARNING: do not change without consulting AVM which this constant affects
pub global MAX_FR_CALLDATA_TO_ALL_ENQUEUED_CALLS: u32 = 16_000;
/**
 * Convention for constant array lengths are mainly divided in 2 classes:
 *  - FUNCTION CALL
 *  - TRANSACTION
 *
 * Agreed convention is to use MAX_XXX_PER_CALL resp. MAX_XXX_PER_TX, where XXX denotes a type of element such as
 * commitment, or nullifier, e.g.,:
 *  - MAX_NULLIFIERS_PER_CALL
 *  - MAX_NOTE_HASHES_PER_TX
 *
 * In the kernel circuits, we accumulate elements such as note hashes and the nullifiers from all functions calls in a
 * transaction. Therefore, we always must have:
 * MAX_XXX_PER_TX >= MAX_XXX_PER_CALL
 *
 * For instance:
 * MAX_NOTE_HASHES_PER_TX >= MAX_NOTE_HASHES_PER_CALL
 * MAX_NULLIFIERS_PER_TX >= MAX_NULLIFIERS_PER_CALL
 *
 */

// docs:start:constants
// "PER CALL" CONSTANTS
pub global MAX_NOTE_HASHES_PER_CALL: u32 = 16;
pub global MAX_NULLIFIERS_PER_CALL: u32 = 16;
pub global MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL: u32 = 5;
pub global MAX_ENQUEUED_CALLS_PER_CALL: u32 = 16;
pub global MAX_L2_TO_L1_MSGS_PER_CALL: u32 = 2;
pub global MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL: u32 = 63;
pub global MAX_PUBLIC_DATA_READS_PER_CALL: u32 = 64;
pub global MAX_NOTE_HASH_READ_REQUESTS_PER_CALL: u32 = 16;
pub global MAX_NULLIFIER_READ_REQUESTS_PER_CALL: u32 = 16;
pub global MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_CALL: u32 = 16;
pub global MAX_L1_TO_L2_MSG_READ_REQUESTS_PER_CALL: u32 = 16;
pub global MAX_KEY_VALIDATION_REQUESTS_PER_CALL: u32 = 16;
pub global MAX_PRIVATE_LOGS_PER_CALL: u32 = 16;
pub global MAX_PUBLIC_LOGS_PER_CALL: u32 = 4;
pub global MAX_CONTRACT_CLASS_LOGS_PER_CALL: u32 = 1;

// TREES RELATED CONSTANTS
pub global ARCHIVE_HEIGHT: u32 = 29;
pub global VK_TREE_HEIGHT: u32 = 6;
pub global PROTOCOL_CONTRACT_TREE_HEIGHT: u32 = 3;
pub global FUNCTION_TREE_HEIGHT: u32 = 5;
pub global NOTE_HASH_TREE_HEIGHT: u32 = 40;
pub global PUBLIC_DATA_TREE_HEIGHT: u32 = 40;
pub global NULLIFIER_TREE_HEIGHT: u32 = 40;
pub global L1_TO_L2_MSG_TREE_HEIGHT: u32 = 39;
pub global ARTIFACT_FUNCTION_TREE_MAX_HEIGHT: u32 = 5;
pub global NULLIFIER_TREE_ID: Field = 0;
pub global NOTE_HASH_TREE_ID: Field = 1;
pub global PUBLIC_DATA_TREE_ID: Field = 2;
pub global L1_TO_L2_MESSAGE_TREE_ID: Field = 3;
pub global ARCHIVE_TREE_ID: Field = 4;

// SUB-TREES RELATED CONSTANTS
pub global NOTE_HASH_SUBTREE_HEIGHT: u32 = 6;
pub global NULLIFIER_SUBTREE_HEIGHT: u32 = 6;
// Deprecated: to be removed after removal of legacy ts trees
pub global PUBLIC_DATA_SUBTREE_HEIGHT: u32 = 6;
pub global L1_TO_L2_MSG_SUBTREE_HEIGHT: u32 = 4;
pub global NOTE_HASH_SUBTREE_SIBLING_PATH_LENGTH: u32 =
    NOTE_HASH_TREE_HEIGHT - NOTE_HASH_SUBTREE_HEIGHT;
pub global NULLIFIER_SUBTREE_SIBLING_PATH_LENGTH: u32 =
    NULLIFIER_TREE_HEIGHT - NULLIFIER_SUBTREE_HEIGHT;
pub global L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH: u32 =
    L1_TO_L2_MSG_TREE_HEIGHT - L1_TO_L2_MSG_SUBTREE_HEIGHT;

// "PER TRANSACTION" CONSTANTS
pub global MAX_NOTE_HASHES_PER_TX: u32 = (1 as u8 << NOTE_HASH_SUBTREE_HEIGHT as u8) as u32;
pub global MAX_NULLIFIERS_PER_TX: u32 = (1 as u8 << NULLIFIER_SUBTREE_HEIGHT as u8) as u32;
pub global MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX: u32 = 8;
pub global MAX_ENQUEUED_CALLS_PER_TX: u32 = 32;
pub global PROTOCOL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX: u32 = 1;
pub global MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX: u32 =
    (1 as u8 << PUBLIC_DATA_SUBTREE_HEIGHT as u8) as u32;
pub global MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX: u32 =
    MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX - PROTOCOL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX;
pub global MAX_PUBLIC_DATA_READS_PER_TX: u32 = 64;
pub global MAX_L2_TO_L1_MSGS_PER_TX: u32 = 8;
pub global MAX_NOTE_HASH_READ_REQUESTS_PER_TX: u32 = 64;
pub global MAX_NULLIFIER_READ_REQUESTS_PER_TX: u32 = 64;
pub global MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX: u32 = 64;
pub global MAX_L1_TO_L2_MSG_READ_REQUESTS_PER_TX: u32 = 64;
// TODO: for large multisends we might run out of key validation requests here but not dealing with this now as
// databus will hopefully make the issue go away.
pub global MAX_KEY_VALIDATION_REQUESTS_PER_TX: u32 = 64;
pub global MAX_PRIVATE_LOGS_PER_TX: u32 = 32;
pub global MAX_PUBLIC_LOGS_PER_TX: u32 = 8;
pub global MAX_CONTRACT_CLASS_LOGS_PER_TX: u32 = 1;
// docs:end:constants

// ROLLUP CONTRACT CONSTANTS - constants used only in l1-contracts
pub global NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP: u32 = 16;

// VK TREE CONSTANTS
pub comptime global PRIVATE_KERNEL_INIT_INDEX: u32 = 0;
pub comptime global PRIVATE_KERNEL_INNER_INDEX: u32 = 1;
pub comptime global PRIVATE_KERNEL_TAIL_INDEX: u32 = 2;
pub comptime global PRIVATE_KERNEL_TAIL_TO_PUBLIC_INDEX: u32 = 3;
pub comptime global TUBE_VK_INDEX: u32 = 4;
pub comptime global AVM_VK_INDEX: u32 = 5;
pub comptime global PRIVATE_BASE_ROLLUP_VK_INDEX: u32 = 6;
pub comptime global PUBLIC_BASE_ROLLUP_VK_INDEX: u32 = 7;
pub comptime global MERGE_ROLLUP_INDEX: u32 = 8;
pub comptime global BLOCK_ROOT_ROLLUP_INDEX: u32 = 9;
pub comptime global BLOCK_ROOT_ROLLUP_SINGLE_TX_INDEX: u32 = 10;
pub comptime global BLOCK_ROOT_ROLLUP_EMPTY_INDEX: u32 = 11;
pub comptime global BLOCK_MERGE_ROLLUP_INDEX: u32 = 12;
pub comptime global ROOT_ROLLUP_INDEX: u32 = 13;
pub comptime global BASE_PARITY_INDEX: u32 = 14;
pub comptime global ROOT_PARITY_INDEX: u32 = 15;
pub comptime global PRIVATE_KERNEL_RESET_INDEX: u32 = 20;
// Important: Do not define indexes after the PRIVATE_KERNEL_RESET_INDEX. They are allocated for the variants of private kernel reset.

// MISC CONSTANTS
pub global FUNCTION_SELECTOR_NUM_BYTES: Field = 4;
// The following is used in immutable state variables to compute an initialization slot whose value is used to
// determine whether a given variable has been initialized (by asserting that the value in the slot is 0).
// The initialization slot is computed by adding the constant below to the variable's storage slot. This constant has
// to be large enough so that it's ensured that it doesn't collide with storage slots of other variables.
pub global INITIALIZATION_SLOT_SEPARATOR: Field = 1000_000_000;
pub global INITIAL_L2_BLOCK_NUM: Field = 1;
pub global FIELDS_PER_BLOB: u32 = 4096;
pub global BLOBS_PER_BLOCK: u32 = 3;
pub global AZTEC_MAX_EPOCH_DURATION: u32 = 48;
// The genesis value is taken from world_state.test.cpp > WorldStateTest.GetInitialTreeInfoForAllTrees
pub global GENESIS_ARCHIVE_ROOT: Field =
    0x0237797d6a2c04d20d4fa06b74482bd970ccd51a43d9b05b57e9b91fa1ae1cae;
// Last 4 bytes of the Poseidon2 hash of 'public_dispatch(Field)'.
pub global PUBLIC_DISPATCH_SELECTOR: Field = 0xd5441b0d;

// CONTRACT CLASS CONSTANTS
pub global MAX_PACKED_PUBLIC_BYTECODE_SIZE_IN_FIELDS: u32 = 3000;
// Bytecode size for private functions is per function, not for the entire contract.
// Note that private functions bytecode includes a mix of acir and brillig.
pub global MAX_PACKED_BYTECODE_SIZE_PER_PRIVATE_FUNCTION_IN_FIELDS: u32 = 3000;
// Same for utility functions: the size is per function.
pub global MAX_PACKED_BYTECODE_SIZE_PER_UTILITY_FUNCTION_IN_FIELDS: u32 = 3000;
// How many fields are on the serialized ClassPrivateFunctionBroadcasted event in addition to MAX_PACKED_BYTECODE_SIZE_PER_PRIVATE_FUNCTION_IN_FIELDS.
pub global REGISTERER_PRIVATE_FUNCTION_BROADCASTED_ADDITIONAL_FIELDS: u32 = 19;
// How many fields are on the serialized ClassUtilityFunctionBroadcasted event in addition to MAX_PACKED_BYTECODE_SIZE_PER_UTILITY_FUNCTION_IN_FIELDS.
pub global REGISTERER_UTILITY_FUNCTION_BROADCASTED_ADDITIONAL_FIELDS: u32 = 12;
// Since we are not yet emitting selectors we'll use this magic value to identify events emitted by the ClassRegisterer.
// This is just a stopgap until we implement proper selectors.
// sha224sum 'struct ContractClassRegistered {contract_class_id: ContractClassId, version: Field, artifact_hash: Field, private_functions_root: Field, packed_public_bytecode: [Field; MAX_PACKED_PUBLIC_BYTECODE_SIZE_IN_FIELDS] }'
pub global REGISTERER_CONTRACT_CLASS_REGISTERED_MAGIC_VALUE: Field =
    0x6999d1e02b08a447a463563453cb36919c9dd7150336fc7c4d2b52f8;
// sha224sum 'struct ClassPrivateFunctionBroadcasted'
pub global REGISTERER_PRIVATE_FUNCTION_BROADCASTED_MAGIC_VALUE: Field =
    0x1b70e95fde0b70adc30496b90a327af6a5e383e028e7a43211a07bcd;
// sha224sum 'struct ClassUtilityFunctionBroadcasted'
pub global REGISTERER_UTILITY_FUNCTION_BROADCASTED_MAGIC_VALUE: Field =
    0xe7af816635466f128568edb04c9fa024f6c87fb9010fdbffa68b3d99;

// CONTRACT INSTANCE CONSTANTS
// sha224sum 'struct ContractInstanceDeployed'
pub global DEPLOYER_CONTRACT_INSTANCE_DEPLOYED_MAGIC_VALUE: Field =
    0x85864497636cf755ae7bde03f267ce01a520981c21c3682aaf82a631;

// sha224sum 'struct ContractInstanceUpdated'
pub global DEPLOYER_CONTRACT_INSTANCE_UPDATED_MAGIC_VALUE: Field =
    0x0e92f9f8a534e858fcf777da206e08b0c620ecf9deb21d13479813f6;

// CANONICAL CONTRACT ADDRESSES
pub global MAX_PROTOCOL_CONTRACTS: u32 = (1 << PROTOCOL_CONTRACT_TREE_HEIGHT as u8) - 1; // Index 0 can't be used.
pub global CANONICAL_AUTH_REGISTRY_ADDRESS: AztecAddress = AztecAddress::from_field(1);
pub global DEPLOYER_CONTRACT_ADDRESS: AztecAddress = AztecAddress::from_field(2);
pub global REGISTERER_CONTRACT_ADDRESS: AztecAddress = AztecAddress::from_field(3);
pub global MULTI_CALL_ENTRYPOINT_ADDRESS: AztecAddress = AztecAddress::from_field(4);
pub global FEE_JUICE_ADDRESS: AztecAddress = AztecAddress::from_field(5);
pub global ROUTER_ADDRESS: AztecAddress = AztecAddress::from_field(6);

// Randomly chosen slot for the bytecode capsule.
pub global REGISTERER_CONTRACT_BYTECODE_CAPSULE_SLOT: Field = 79025834455612;

// Slot of the balances map to be hashed with an AztecAddress (map key) to get an actual storage slot.
pub global FEE_JUICE_BALANCES_SLOT: u32 = 1;

// Slot of the updated_class_ids map to be hashed with an AztecAddress (map key) to get an actual storage slot.
pub global UPDATED_CLASS_IDS_SLOT: u32 = 1;

// CANONICAL DEFAULT KEYS
// This below are:
// "az_null_npk"
// "az_null_ivpk"
// "az_null_ovpk"
// "az_null_tpk"
// as bytes, hashed to curve using grumpkin::g1::affine_element::hash_to_curve(<X>, 0);
pub global DEFAULT_NPK_M_X: Field =
    0x01498945581e0eb9f8427ad6021184c700ef091d570892c437d12c7d90364bbd;
pub global DEFAULT_NPK_M_Y: Field =
    0x170ae506787c5c43d6ca9255d571c10fa9ffa9d141666e290c347c5c9ab7e344;
pub global DEFAULT_IVPK_M_X: Field =
    0x00c044b05b6ca83b9c2dbae79cc1135155956a64e136819136e9947fe5e5866c;
pub global DEFAULT_IVPK_M_Y: Field =
    0x1c1f0ca244c7cd46b682552bff8ae77dea40b966a71de076ec3b7678f2bdb151;
pub global DEFAULT_OVPK_M_X: Field =
    0x1b00316144359e9a3ec8e49c1cdb7eeb0cedd190dfd9dc90eea5115aa779e287;
pub global DEFAULT_OVPK_M_Y: Field =
    0x080ffc74d7a8b0bccb88ac11f45874172f3847eb8b92654aaa58a3d2b8dc7833;
pub global DEFAULT_TPK_M_X: Field =
    0x019c111f36ad3fc1d9b7a7a14344314d2864b94f030594cd67f753ef774a1efb;
pub global DEFAULT_TPK_M_Y: Field =
    0x2039907fe37f08d10739255141bb066c506a12f7d1e8dfec21abc58494705b6f;

// LENGTH OF STRUCTS SERIALIZED TO FIELDS
pub global AZTEC_ADDRESS_LENGTH: u32 = 1;
pub global GAS_FEES_LENGTH: u32 = 2;
pub global GAS_LENGTH: u32 = 2;
pub global GAS_SETTINGS_LENGTH: u32 = GAS_LENGTH /* gas_limits */
    + GAS_LENGTH /* teardown_gas_limits */
    + GAS_FEES_LENGTH /* max_fees_per_gas */
    + GAS_FEES_LENGTH /* max_priority_fees_per_gas */;
pub global CALL_CONTEXT_LENGTH: u32 = 4;
pub global CONTENT_COMMITMENT_LENGTH: u32 = 4;
pub global CONTRACT_INSTANCE_LENGTH: u32 = 16;
pub global CONTRACT_STORAGE_READ_LENGTH: u32 = 3;
pub global CONTRACT_STORAGE_UPDATE_REQUEST_LENGTH: u32 = 3;
pub global ETH_ADDRESS_LENGTH: u32 = 1;
pub global FUNCTION_DATA_LENGTH: u32 = 2;
pub global FUNCTION_LEAF_PREIMAGE_LENGTH: u32 = 5;
pub global GLOBAL_VARIABLES_LENGTH: u32 = 7 + GAS_FEES_LENGTH;
pub global APPEND_ONLY_TREE_SNAPSHOT_LENGTH: u32 = 2;
pub global APPEND_ONLY_TREE_SNAPSHOT_LENGTH_BYTES: u32 = 36;
pub global SPONGE_BLOB_LENGTH: u32 = 11;
pub global BLS12_FR_LIMBS: u32 = 3; // TODO(MW): get this from bignum
pub global BLS12_FQ_LIMBS: u32 = 4; // TODO(MW): get this from bignum
pub global BLS12_POINT: u32 = 2 * BLS12_FQ_LIMBS;
pub global BLOB_ACCUMULATOR_PUBLIC_INPUTS: u32 = 3 /* v_acc, z_acc, and gamma_acc */
    + 2 * BLS12_FR_LIMBS /* y_acc and gamma^i */
    + BLS12_POINT /* c_acc */;
pub global BLOCK_BLOB_PUBLIC_INPUTS: u32 = 2 * BLOB_ACCUMULATOR_PUBLIC_INPUTS /* start and end accumulators */
    + 1 /* z */
    + BLS12_FR_LIMBS /* gamma */;
pub global BLOB_PUBLIC_INPUTS: u32 = 6;
pub global BLOB_PUBLIC_INPUTS_BYTES: u32 = 112; // = commitment (48) + z (32) + y (32)
pub global L1_TO_L2_MESSAGE_LENGTH: u32 = 6;
pub global L2_TO_L1_MESSAGE_LENGTH: u32 = 3;
pub global SCOPED_L2_TO_L1_MESSAGE_LENGTH: u32 = L2_TO_L1_MESSAGE_LENGTH + 1;
pub global MAX_BLOCK_NUMBER_LENGTH: u32 = 2; // 1 for the option flag, 1 for the value
pub global KEY_VALIDATION_REQUEST_LENGTH: u32 = 4;
pub global KEY_VALIDATION_REQUEST_AND_GENERATOR_LENGTH: u32 = KEY_VALIDATION_REQUEST_LENGTH + 1;
pub global SCOPED_KEY_VALIDATION_REQUEST_AND_GENERATOR_LENGTH: u32 =
    KEY_VALIDATION_REQUEST_AND_GENERATOR_LENGTH + 1;
pub global PARTIAL_STATE_REFERENCE_LENGTH: u32 = 6;
pub global READ_REQUEST_LENGTH: u32 = 2;
pub global TREE_LEAF_READ_REQUEST_LENGTH: u32 = 2;
pub global PRIVATE_LOG_SIZE_IN_FIELDS: u32 = 18; // This is currently affected by the size of the log overhead defined in aztec-nr/aztec/src/messages/payload.nr.
pub global PRIVATE_LOG_LENGTH: u32 = PRIVATE_LOG_SIZE_IN_FIELDS + 1 /* length */;
pub global PRIVATE_LOG_DATA_LENGTH: u32 = PRIVATE_LOG_LENGTH
    + 1 /* note_hash_counter */
    + 1 /* counter */;
pub global SCOPED_PRIVATE_LOG_DATA_LENGTH: u32 = PRIVATE_LOG_DATA_LENGTH + 1 /* contract_address */;
pub global PUBLIC_LOG_SIZE_IN_FIELDS: u32 = 13;
pub global PUBLIC_LOG_LENGTH: u32 = PUBLIC_LOG_SIZE_IN_FIELDS
    + 1 /* length */
    + 1 /* contract_address */;
// TODO(MW): add new constant for this - the below is just the largest combination atm
pub global CONTRACT_CLASS_LOG_SIZE_IN_FIELDS: u32 = MAX_PACKED_BYTECODE_SIZE_PER_PRIVATE_FUNCTION_IN_FIELDS
    + REGISTERER_PRIVATE_FUNCTION_BROADCASTED_ADDITIONAL_FIELDS;
pub global CONTRACT_CLASS_LOG_LENGTH: u32 = CONTRACT_CLASS_LOG_SIZE_IN_FIELDS
    + 1 /* length */
    + 1 /* contract_address */;
pub global LOG_HASH_LENGTH: u32 = 1 /* value */ + 1 /* length */;
pub global COUNTED_LOG_HASH_LENGTH: u32 = LOG_HASH_LENGTH + 1 /* counter */;
pub global SCOPED_LOG_HASH_LENGTH: u32 = LOG_HASH_LENGTH + 1 /* contract_address */;
pub global SCOPED_COUNTED_LOG_HASH_LENGTH: u32 = SCOPED_LOG_HASH_LENGTH + 1 /* counter */;
pub global NOTE_HASH_LENGTH: u32 = 2;
pub global SCOPED_NOTE_HASH_LENGTH: u32 = NOTE_HASH_LENGTH + 1;
pub global NULLIFIER_LENGTH: u32 = 3;
pub global SCOPED_NULLIFIER_LENGTH: u32 = NULLIFIER_LENGTH + 1;
pub global PUBLIC_DATA_WRITE_LENGTH: u32 = 2;
pub global PUBLIC_CALL_STACK_ITEM_COMPRESSED_LENGTH: u32 =
    AZTEC_ADDRESS_LENGTH + CALL_CONTEXT_LENGTH + 3 + 2 * GAS_LENGTH;
pub global PRIVATE_CALL_REQUEST_LENGTH: u32 = CALL_CONTEXT_LENGTH + 4;
pub global PUBLIC_CALL_REQUEST_LENGTH: u32 = AZTEC_ADDRESS_LENGTH /* msg_sender */
    + AZTEC_ADDRESS_LENGTH /* contract_address */
    + 1 /* is_static_call */
    + 1 /* calldata_hash */;
pub global COUNTED_PUBLIC_CALL_REQUEST_LENGTH: u32 = PUBLIC_CALL_REQUEST_LENGTH + 1;
pub global PUBLIC_INNER_CALL_REQUEST_LENGTH: u32 =
    PUBLIC_CALL_STACK_ITEM_COMPRESSED_LENGTH + 1 /* counter */;
pub global ROLLUP_VALIDATION_REQUESTS_LENGTH: u32 = MAX_BLOCK_NUMBER_LENGTH;
pub global STATE_REFERENCE_LENGTH: u32 =
    APPEND_ONLY_TREE_SNAPSHOT_LENGTH + PARTIAL_STATE_REFERENCE_LENGTH;
pub global TREE_SNAPSHOTS_LENGTH: u32 = APPEND_ONLY_TREE_SNAPSHOT_LENGTH * 4;
pub global TX_CONTEXT_LENGTH: u32 = 2 + GAS_SETTINGS_LENGTH;
pub global TX_REQUEST_LENGTH: u32 = 2 + TX_CONTEXT_LENGTH + FUNCTION_DATA_LENGTH;
pub global TOTAL_FEES_LENGTH: u32 = 1;
pub global TOTAL_MANA_USED_LENGTH: u32 = 1;
pub global BLOCK_HEADER_LENGTH: u32 = APPEND_ONLY_TREE_SNAPSHOT_LENGTH
    + CONTENT_COMMITMENT_LENGTH
    + STATE_REFERENCE_LENGTH
    + GLOBAL_VARIABLES_LENGTH
    + TOTAL_FEES_LENGTH
    + TOTAL_MANA_USED_LENGTH;
// Global variables are all 32 bytes, apart from coinbase which is 20, hence -12. A state reference is just 4 snapshots.
pub global BLOCK_HEADER_LENGTH_BYTES: u32 = APPEND_ONLY_TREE_SNAPSHOT_LENGTH_BYTES
    + 32 * CONTENT_COMMITMENT_LENGTH
    + 4 * APPEND_ONLY_TREE_SNAPSHOT_LENGTH_BYTES
    + 32 * GLOBAL_VARIABLES_LENGTH
    - 12
    + 32 * TOTAL_FEES_LENGTH
    + 32 * TOTAL_MANA_USED_LENGTH;
pub global PROPOSED_BLOCK_HEADER_LENGTH: u32 = 1 /* last_archive_root */
    + CONTENT_COMMITMENT_LENGTH
    + 1 /* slot_number */
    + 1 /* timestamp */
    + 1 /* coinbase */
    + 1 /* fee_recipient */
    + GAS_FEES_LENGTH
    + TOTAL_MANA_USED_LENGTH;
pub global PROPOSED_BLOCK_HEADER_LENGTH_BYTES: u32 = (PROPOSED_BLOCK_HEADER_LENGTH - 2) * 32
    + 8 /* timestamp */
    + 20 /* coinbase */;
pub global PRIVATE_CIRCUIT_PUBLIC_INPUTS_LENGTH: u32 = CALL_CONTEXT_LENGTH
    + 4
    + MAX_BLOCK_NUMBER_LENGTH
    + (READ_REQUEST_LENGTH * MAX_NOTE_HASH_READ_REQUESTS_PER_CALL)
    + (READ_REQUEST_LENGTH * MAX_NULLIFIER_READ_REQUESTS_PER_CALL)
    + (KEY_VALIDATION_REQUEST_AND_GENERATOR_LENGTH * MAX_KEY_VALIDATION_REQUESTS_PER_CALL)
    + (NOTE_HASH_LENGTH * MAX_NOTE_HASHES_PER_CALL)
    + (NULLIFIER_LENGTH * MAX_NULLIFIERS_PER_CALL)
    + (PRIVATE_CALL_REQUEST_LENGTH * MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL)
    + (COUNTED_PUBLIC_CALL_REQUEST_LENGTH * MAX_ENQUEUED_CALLS_PER_CALL)
    + PUBLIC_CALL_REQUEST_LENGTH
    + (L2_TO_L1_MESSAGE_LENGTH * MAX_L2_TO_L1_MSGS_PER_CALL)
    + 2
    + (PRIVATE_LOG_DATA_LENGTH * MAX_PRIVATE_LOGS_PER_CALL)
    + (COUNTED_LOG_HASH_LENGTH * MAX_CONTRACT_CLASS_LOGS_PER_CALL)
    + BLOCK_HEADER_LENGTH
    + TX_CONTEXT_LENGTH;
pub global PRIVATE_CONTEXT_INPUTS_LENGTH: u32 =
    CALL_CONTEXT_LENGTH + BLOCK_HEADER_LENGTH + TX_CONTEXT_LENGTH + 1;
pub global FEE_RECIPIENT_LENGTH: u32 = 2;

pub global PAIRING_POINTS_SIZE: u32 = 16;
pub global IPA_CLAIM_SIZE: u32 = 10;

pub global SCOPED_READ_REQUEST_LEN: u32 = READ_REQUEST_LENGTH + 1;
pub global PUBLIC_DATA_READ_LENGTH: u32 = 3;
pub global PRIVATE_VALIDATION_REQUESTS_LENGTH: u32 = ROLLUP_VALIDATION_REQUESTS_LENGTH
    + (SCOPED_READ_REQUEST_LEN * MAX_NOTE_HASH_READ_REQUESTS_PER_TX)
    + (SCOPED_READ_REQUEST_LEN * MAX_NULLIFIER_READ_REQUESTS_PER_TX)
    + (SCOPED_KEY_VALIDATION_REQUEST_AND_GENERATOR_LENGTH * MAX_KEY_VALIDATION_REQUESTS_PER_TX)
    + 2;

pub global PRIVATE_TO_ROLLUP_ACCUMULATED_DATA_LENGTH: u32 = MAX_NOTE_HASHES_PER_TX
    + MAX_NULLIFIERS_PER_TX
    + (MAX_L2_TO_L1_MSGS_PER_TX * SCOPED_L2_TO_L1_MESSAGE_LENGTH)
    + (PRIVATE_LOG_LENGTH * MAX_PRIVATE_LOGS_PER_TX)
    + (SCOPED_LOG_HASH_LENGTH * MAX_CONTRACT_CLASS_LOGS_PER_TX);

pub global TX_CONSTANT_DATA_LENGTH: u32 = BLOCK_HEADER_LENGTH
    + TX_CONTEXT_LENGTH
    + 1 /* vk_tree_root */
    + 1 /* protocol_contract_tree_root */;
pub global COMBINED_CONSTANT_DATA_LENGTH: u32 = TX_CONSTANT_DATA_LENGTH + GLOBAL_VARIABLES_LENGTH;

pub global PRIVATE_ACCUMULATED_DATA_LENGTH: u32 = (SCOPED_NOTE_HASH_LENGTH * MAX_NOTE_HASHES_PER_TX)
    + (SCOPED_NULLIFIER_LENGTH * MAX_NULLIFIERS_PER_TX)
    + (MAX_L2_TO_L1_MSGS_PER_TX * SCOPED_L2_TO_L1_MESSAGE_LENGTH)
    + (SCOPED_PRIVATE_LOG_DATA_LENGTH * MAX_PRIVATE_LOGS_PER_TX)
    + (SCOPED_COUNTED_LOG_HASH_LENGTH * MAX_CONTRACT_CLASS_LOGS_PER_TX)
    + (PRIVATE_CALL_REQUEST_LENGTH * MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX)
    + (COUNTED_PUBLIC_CALL_REQUEST_LENGTH * MAX_ENQUEUED_CALLS_PER_TX);
pub global PRIVATE_KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH: u32 = TX_CONSTANT_DATA_LENGTH
    + 1 /* min_revertible_side_effect_counter */
    + PRIVATE_VALIDATION_REQUESTS_LENGTH
    + PRIVATE_ACCUMULATED_DATA_LENGTH
    + PUBLIC_CALL_REQUEST_LENGTH
    + AZTEC_ADDRESS_LENGTH
    + 1 /* is private only */
    + 1 /* claimed_first_nullifier */;

pub global PRIVATE_TO_PUBLIC_ACCUMULATED_DATA_LENGTH: u32 = MAX_NOTE_HASHES_PER_TX
    + MAX_NULLIFIERS_PER_TX
    + (MAX_L2_TO_L1_MSGS_PER_TX * SCOPED_L2_TO_L1_MESSAGE_LENGTH)
    + (MAX_PRIVATE_LOGS_PER_TX * PRIVATE_LOG_LENGTH)
    + (MAX_CONTRACT_CLASS_LOGS_PER_TX * SCOPED_LOG_HASH_LENGTH)
    + (MAX_ENQUEUED_CALLS_PER_TX * PUBLIC_CALL_REQUEST_LENGTH);

pub global PRIVATE_TO_AVM_ACCUMULATED_DATA_LENGTH: u32 = MAX_NOTE_HASHES_PER_TX
    + MAX_NULLIFIERS_PER_TX
    + (MAX_L2_TO_L1_MSGS_PER_TX * SCOPED_L2_TO_L1_MESSAGE_LENGTH);
pub global NUM_PRIVATE_TO_AVM_ACCUMULATED_DATA_ARRAYS: u32 = 3;

pub global AVM_ACCUMULATED_DATA_LENGTH: u32 = MAX_NOTE_HASHES_PER_TX
    + MAX_NULLIFIERS_PER_TX
    + (MAX_L2_TO_L1_MSGS_PER_TX * SCOPED_L2_TO_L1_MESSAGE_LENGTH)
    + (MAX_PUBLIC_LOGS_PER_TX * PUBLIC_LOG_LENGTH)
    + (MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX * PUBLIC_DATA_WRITE_LENGTH);

pub global PRIVATE_TO_PUBLIC_KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH: u32 = TX_CONSTANT_DATA_LENGTH
    + ROLLUP_VALIDATION_REQUESTS_LENGTH
    + PRIVATE_TO_PUBLIC_ACCUMULATED_DATA_LENGTH /* non_revertible_accumulated_data */
    + PRIVATE_TO_PUBLIC_ACCUMULATED_DATA_LENGTH /* revertible_accumulated_data */
    + PUBLIC_CALL_REQUEST_LENGTH /* public_teardown_call_request */
    + GAS_LENGTH /* gas_used */
    + AZTEC_ADDRESS_LENGTH /* fee_payer */;

pub global PRIVATE_TO_ROLLUP_KERNEL_CIRCUIT_PUBLIC_INPUTS_LENGTH: u32 = ROLLUP_VALIDATION_REQUESTS_LENGTH
    + PRIVATE_TO_ROLLUP_ACCUMULATED_DATA_LENGTH
    + TX_CONSTANT_DATA_LENGTH
    + GAS_LENGTH /* gas_used */
    + AZTEC_ADDRESS_LENGTH;

pub global AVM_CIRCUIT_PUBLIC_INPUTS_LENGTH: u32 = GLOBAL_VARIABLES_LENGTH
    + TREE_SNAPSHOTS_LENGTH /* start_tree_snapshots */
    + GAS_LENGTH /* start_gas_used */
    + GAS_SETTINGS_LENGTH
    + 1 /* fee_payer */
    + (MAX_ENQUEUED_CALLS_PER_TX * PUBLIC_CALL_REQUEST_LENGTH) /* public_setup_call_requests */
    + (MAX_ENQUEUED_CALLS_PER_TX * PUBLIC_CALL_REQUEST_LENGTH) /* public_app_logic_call_requests */
    + PUBLIC_CALL_REQUEST_LENGTH /* public_teardown_call_request */
    + NUM_PRIVATE_TO_AVM_ACCUMULATED_DATA_ARRAYS /* previous_non_revertible_accumulated_data_array_lengths */
    + NUM_PRIVATE_TO_AVM_ACCUMULATED_DATA_ARRAYS /* previous_revertible_accumulated_data_array_lengths */
    + PRIVATE_TO_AVM_ACCUMULATED_DATA_LENGTH /* previous_non_revertible_accumulated_data */
    + PRIVATE_TO_AVM_ACCUMULATED_DATA_LENGTH /* previous_revertible_accumulated_data */
    + TREE_SNAPSHOTS_LENGTH /* end_tree_snapshots */
    + GAS_LENGTH /* end_gas_used */
    + AVM_ACCUMULATED_DATA_LENGTH
    + 1 /* transaction_fee */
    + 1 /* reverted */;

pub global CONSTANT_ROLLUP_DATA_LENGTH: u32 = APPEND_ONLY_TREE_SNAPSHOT_LENGTH
    + 1 /* vk_tree_root */
    + 1 /* protocol_contract_tree_root */
    + GLOBAL_VARIABLES_LENGTH;

// + 5 for rollup_type, num_txs, out_hash, accumulated_fees, accumulated_mana_used
pub global BASE_OR_MERGE_PUBLIC_INPUTS_LENGTH: u32 = CONSTANT_ROLLUP_DATA_LENGTH
    + PARTIAL_STATE_REFERENCE_LENGTH
    + PARTIAL_STATE_REFERENCE_LENGTH
    + 2 * SPONGE_BLOB_LENGTH
    + 5;
pub global BLOCK_ROOT_OR_BLOCK_MERGE_PUBLIC_INPUTS_LENGTH: u32 = 2
    * APPEND_ONLY_TREE_SNAPSHOT_LENGTH
    + 2 * GLOBAL_VARIABLES_LENGTH
    + 1 /* out_hash */
    + AZTEC_MAX_EPOCH_DURATION /* proposedBlockHeaderHashes */
    + AZTEC_MAX_EPOCH_DURATION * FEE_RECIPIENT_LENGTH
    + 1 /* vk_tree_root */
    + 1 /* protocol_contract_tree_root */
    + 1 /* prover_id */
    + BLOCK_BLOB_PUBLIC_INPUTS /* blob_public_inputs */;
pub global ROOT_ROLLUP_PUBLIC_INPUTS_LENGTH: u32 = APPEND_ONLY_TREE_SNAPSHOT_LENGTH /* previous_archive */
    + APPEND_ONLY_TREE_SNAPSHOT_LENGTH /* end_archive */
    + 1 /* end_timestamp */
    + 1 /* end_block_number */
    + 1 /* out_hash */
    + 1 /* chain_id */
    + 1 /* version */
    + 1 /* vk_tree_root */
    + 1 /* protocol_contract_tree_root */
    + 1 /* prover_id */
    + AZTEC_MAX_EPOCH_DURATION /* proposedBlockHeaderHashes */
    + AZTEC_MAX_EPOCH_DURATION * FEE_RECIPIENT_LENGTH /* fees */
    + 1 /* blob_commitments_hash */
    + 1 /* z */
    + BLS12_FR_LIMBS /* y */
    + 2 /* c */;
pub global GET_NOTES_ORACLE_RETURN_LENGTH: u32 = 674;
pub global NOTE_HASHES_NUM_BYTES_PER_BASE_ROLLUP: u32 = 32 * MAX_NOTE_HASHES_PER_TX;
pub global NULLIFIERS_NUM_BYTES_PER_BASE_ROLLUP: u32 = 32 * MAX_NULLIFIERS_PER_TX;
pub global PUBLIC_DATA_WRITES_NUM_BYTES_PER_BASE_ROLLUP: u32 =
    64 * MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX; // 1 write is 64 bytes
pub global PRIVATE_LOGS_NUM_BYTES_PER_BASE_ROLLUP: u32 =
    32 * PRIVATE_LOG_SIZE_IN_FIELDS * MAX_PRIVATE_LOGS_PER_TX;
pub global CONTRACTS_NUM_BYTES_PER_BASE_ROLLUP: Field = 32;
pub global CONTRACT_DATA_NUM_BYTES_PER_BASE_ROLLUP: Field = 64;
pub global CONTRACT_DATA_NUM_BYTES_PER_BASE_ROLLUP_UNPADDED: Field = 52;
pub global L2_TO_L1_MSGS_NUM_BYTES_PER_BASE_ROLLUP: Field = 256;
pub global LOGS_HASHES_NUM_BYTES_PER_BASE_ROLLUP: Field = 64;
pub global NUM_MSGS_PER_BASE_PARITY: u32 = 4;
// pub global NUM_BASE_PARITY_PER_ROOT_PARITY: u32 = NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP / NUM_MSGS_PER_BASE_PARITY;
// FIX: Sadly, writing this as above causes a type error in type_conversion.ts.
pub global NUM_BASE_PARITY_PER_ROOT_PARITY: u32 = 4;

// Lengths of the different types of proofs in fields
pub global RECURSIVE_PROOF_LENGTH: u32 = 456;
pub global NESTED_RECURSIVE_PROOF_LENGTH: u32 = RECURSIVE_PROOF_LENGTH;
pub global IPA_PROOF_LENGTH: u32 = 69;
pub global RECURSIVE_ROLLUP_HONK_PROOF_LENGTH: u32 =
    RECURSIVE_PROOF_LENGTH + IPA_CLAIM_SIZE + IPA_PROOF_LENGTH;
pub global NESTED_RECURSIVE_ROLLUP_HONK_PROOF_LENGTH: u32 = RECURSIVE_ROLLUP_HONK_PROOF_LENGTH;
pub global TUBE_PROOF_LENGTH: u32 = RECURSIVE_ROLLUP_HONK_PROOF_LENGTH; // in the future these can differ

pub global HONK_VERIFICATION_KEY_LENGTH_IN_FIELDS: u32 = 112; // size of an Ultra verification key
pub global ROLLUP_HONK_VERIFICATION_KEY_LENGTH_IN_FIELDS: u32 = 113; // size of an Ultra Rollup verification key
pub global CLIENT_IVC_VERIFICATION_KEY_LENGTH_IN_FIELDS: u32 = 127; // size of a Mega verification key
// VK is composed of
// - circuit size encoded as a fr field element (32 bytes)
// - num of inputs encoded as a fr field element (32 bytes)
// - 21 affine elements (curve base field fq) encoded as fr elements takes (21 * 4 * 32 bytes)
// 21 above refers to the constant AvmFlavor::NUM_PRECOMPUTED_ENTITIES
pub global AVM_VERIFICATION_KEY_LENGTH_IN_FIELDS: u32 = 2 + 21 * 4;

// Setting limits for MAX_PUBLIC_CALLS_TO_UNIQUE_CONTRACT_CLASS_IDS
// This value is determined by the length of the AVM trace and the MAX_PUBLIC_BYTECODE_SIZE_IN_BYTES
// (i.e. 2^21 / MAX_PUBLIC_BYTECODE_SIZE_IN_BYTES ==> 2^21 / 96,000 = 21
pub global MAX_PUBLIC_BYTECODE_SIZE_IN_BYTES: u32 = MAX_PACKED_PUBLIC_BYTECODE_SIZE_IN_FIELDS * 32;
pub global MAX_PUBLIC_CALLS_TO_UNIQUE_CONTRACT_CLASS_IDS: u32 = 21;

// Note hash generator index which can be used by custom implementations of NoteHash::compute_note_hash
pub global GENERATOR_INDEX__NOTE_HASH: u32 = 1;
pub global GENERATOR_INDEX__NOTE_HASH_NONCE: u32 = 2;
pub global GENERATOR_INDEX__UNIQUE_NOTE_HASH: u32 = 3;
pub global GENERATOR_INDEX__SILOED_NOTE_HASH: u32 = 4;
pub global GENERATOR_INDEX__MESSAGE_NULLIFIER: u32 = 5;
pub global GENERATOR_INDEX__INITIALIZATION_NULLIFIER: u32 = 6;
pub global GENERATOR_INDEX__OUTER_NULLIFIER: u32 = 7;
pub global GENERATOR_INDEX__PUBLIC_DATA_READ: u32 = 8;
pub global GENERATOR_INDEX__PUBLIC_DATA_UPDATE_REQUEST: u32 = 9;
pub global GENERATOR_INDEX__FUNCTION_DATA: u32 = 10;
pub global GENERATOR_INDEX__FUNCTION_LEAF: u32 = 11;
pub global GENERATOR_INDEX__CONTRACT_DEPLOYMENT_DATA: u32 = 12;
pub global GENERATOR_INDEX__CONSTRUCTOR: u32 = 13;
pub global GENERATOR_INDEX__CONSTRUCTOR_ARGS: u32 = 14;
pub global GENERATOR_INDEX__CONTRACT_ADDRESS_V1: u32 = 15;
pub global GENERATOR_INDEX__CONTRACT_LEAF: u32 = 16;
pub global GENERATOR_INDEX__CALL_CONTEXT: u32 = 17;
pub global GENERATOR_INDEX__CALL_STACK_ITEM: u32 = 18;
pub global GENERATOR_INDEX__CALL_STACK_ITEM_2: u32 = 19;
pub global GENERATOR_INDEX__SECRET_HASH: u32 = 20;
pub global GENERATOR_INDEX__L2_TO_L1_MSG: u32 = 21;
pub global GENERATOR_INDEX__TX_CONTEXT: u32 = 22;
pub global GENERATOR_INDEX__PUBLIC_LEAF_INDEX: u32 = 23;
pub global GENERATOR_INDEX__PUBLIC_DATA_LEAF: u32 = 24;
pub global GENERATOR_INDEX__SIGNED_TX_REQUEST: u32 = 25;
pub global GENERATOR_INDEX__GLOBAL_VARIABLES: u32 = 26;
pub global GENERATOR_INDEX__PARTIAL_ADDRESS: u32 = 27;
pub global GENERATOR_INDEX__BLOCK_HASH: u32 = 28;
pub global GENERATOR_INDEX__SIDE_EFFECT: u32 = 29;
pub global GENERATOR_INDEX__FEE_PAYLOAD: u32 = 30;
pub global GENERATOR_INDEX__COMBINED_PAYLOAD: u32 = 31;
pub global GENERATOR_INDEX__TX_NULLIFIER: u32 = 32;
pub global GENERATOR_INDEX__TX_REQUEST: u32 = 33;
pub global GENERATOR_INDEX__SIGNATURE_PAYLOAD: u32 = 34;
pub global GENERATOR_INDEX__VK: u32 = 41;
pub global GENERATOR_INDEX__PUBLIC_CALLDATA: u32 = 43;
pub global GENERATOR_INDEX__FUNCTION_ARGS: u32 = 44;
pub global GENERATOR_INDEX__AUTHWIT_INNER: u32 = 45;
pub global GENERATOR_INDEX__AUTHWIT_OUTER: u32 = 46;
pub global GENERATOR_INDEX__AUTHWIT_NULLIFIER: u32 = 47;
// Key related generators follow
pub global GENERATOR_INDEX__NSK_M: u32 = 48;
pub global GENERATOR_INDEX__IVSK_M: u32 = 49;
pub global GENERATOR_INDEX__OVSK_M: u32 = 50;
pub global GENERATOR_INDEX__TSK_M: u32 = 51;
pub global GENERATOR_INDEX__PUBLIC_KEYS_HASH: u32 = 52;
pub global GENERATOR_INDEX__NOTE_NULLIFIER: u32 = 53;
pub global GENERATOR_INDEX__SYMMETRIC_KEY: u8 = 54;
pub global GENERATOR_INDEX__SYMMETRIC_KEY_2: u8 = 55;
pub global GENERATOR_INDEX__PUBLIC_TX_HASH: u32 = 56;
pub global GENERATOR_INDEX__PRIVATE_TX_HASH: u32 = 57;

// AVM memory tags
pub global MEM_TAG_FF: Field = 0;
pub global MEM_TAG_U1: Field = 1;
pub global MEM_TAG_U8: Field = 2;
pub global MEM_TAG_U16: Field = 3;
pub global MEM_TAG_U32: Field = 4;
pub global MEM_TAG_U64: Field = 5;
pub global MEM_TAG_U128: Field = 6;

// AvmCircuitPublicInputs - row indices
// TODO(dbanks12): could we somehow use Noir macros to generate these?
// TODO(dbanks12): could we somehow generate these from the actual struct? Or no, because codegen can't use that?

// Global Variables
// start alias
pub global AVM_PUBLIC_INPUTS_GLOBAL_VARIABLES_ROW_IDX: u32 = 0;
// members
pub global AVM_PUBLIC_INPUTS_GLOBAL_VARIABLES_CHAIN_ID_ROW_IDX: u32 =
    AVM_PUBLIC_INPUTS_GLOBAL_VARIABLES_ROW_IDX;
pub global AVM_PUBLIC_INPUTS_GLOBAL_VARIABLES_VERSION_ROW_IDX: u32 =
    AVM_PUBLIC_INPUTS_GLOBAL_VARIABLES_CHAIN_ID_ROW_IDX + 1;
pub global AVM_PUBLIC_INPUTS_GLOBAL_VARIABLES_BLOCK_NUMBER_ROW_IDX: u32 =
    AVM_PUBLIC_INPUTS_GLOBAL_VARIABLES_VERSION_ROW_IDX + 1;
pub global AVM_PUBLIC_INPUTS_GLOBAL_VARIABLES_SLOT_NUMBER_ROW_IDX: u32 =
    AVM_PUBLIC_INPUTS_GLOBAL_VARIABLES_BLOCK_NUMBER_ROW_IDX + 1;
pub global AVM_PUBLIC_INPUTS_GLOBAL_VARIABLES_TIMESTAMP_ROW_IDX: u32 =
    AVM_PUBLIC_INPUTS_GLOBAL_VARIABLES_SLOT_NUMBER_ROW_IDX + 1;
pub global AVM_PUBLIC_INPUTS_GLOBAL_VARIABLES_COINBASE_ROW_IDX: u32 =
    AVM_PUBLIC_INPUTS_GLOBAL_VARIABLES_TIMESTAMP_ROW_IDX + 1;
pub global AVM_PUBLIC_INPUTS_GLOBAL_VARIABLES_FEE_RECIPIENT_ROW_IDX: u32 =
    AVM_PUBLIC_INPUTS_GLOBAL_VARIABLES_COINBASE_ROW_IDX + 1;
pub global AVM_PUBLIC_INPUTS_GLOBAL_VARIABLES_GAS_FEES_ROW_IDX: u32 =
    AVM_PUBLIC_INPUTS_GLOBAL_VARIABLES_FEE_RECIPIENT_ROW_IDX + 1;
// Note: gas fees ^, despite being 2 entries, is split across 2 columns. So wee only bump by 1 (below) to progress.

// Start Tree Snapshots
// start alias
pub global AVM_PUBLIC_INPUTS_START_TREE_SNAPSHOTS_ROW_IDX: u32 =
    AVM_PUBLIC_INPUTS_GLOBAL_VARIABLES_GAS_FEES_ROW_IDX + 1;
// Note: each tree snapshot ^, despite being 2 entries (root, index), is split across 2 columns. So we only bump by 1 (below) to progress.
// members
pub global AVM_PUBLIC_INPUTS_START_TREE_SNAPSHOTS_L1_TO_L2_MESSAGE_TREE_ROW_IDX: u32 =
    AVM_PUBLIC_INPUTS_START_TREE_SNAPSHOTS_ROW_IDX;
pub global AVM_PUBLIC_INPUTS_START_TREE_SNAPSHOTS_NOTE_HASH_TREE_ROW_IDX: u32 =
    AVM_PUBLIC_INPUTS_START_TREE_SNAPSHOTS_L1_TO_L2_MESSAGE_TREE_ROW_IDX + 1;
pub global AVM_PUBLIC_INPUTS_START_TREE_SNAPSHOTS_NULLIFIER_TREE_ROW_IDX: u32 =
    AVM_PUBLIC_INPUTS_START_TREE_SNAPSHOTS_NOTE_HASH_TREE_ROW_IDX + 1;
pub global AVM_PUBLIC_INPUTS_START_TREE_SNAPSHOTS_PUBLIC_DATA_TREE_ROW_IDX: u32 =
    AVM_PUBLIC_INPUTS_START_TREE_SNAPSHOTS_NULLIFIER_TREE_ROW_IDX + 1;

// Start Gas Used
pub global AVM_PUBLIC_INPUTS_START_GAS_USED_ROW_IDX: u32 =
    AVM_PUBLIC_INPUTS_START_TREE_SNAPSHOTS_PUBLIC_DATA_TREE_ROW_IDX + 1;
// Note: gas used ^, despite being 2 entries, is split across 2 columns. So we only bump by 1 (below) to progress.

// Gas Settings
pub global AVM_PUBLIC_INPUTS_GAS_SETTINGS_ROW_IDX: u32 =
    AVM_PUBLIC_INPUTS_START_GAS_USED_ROW_IDX + 1;
pub global AVM_PUBLIC_INPUTS_GAS_SETTINGS_GAS_LIMITS_ROW_IDX: u32 =
    AVM_PUBLIC_INPUTS_GAS_SETTINGS_ROW_IDX;
pub global AVM_PUBLIC_INPUTS_GAS_SETTINGS_TEARDOWN_GAS_LIMITS_ROW_IDX: u32 =
    AVM_PUBLIC_INPUTS_GAS_SETTINGS_GAS_LIMITS_ROW_IDX + 1;
pub global AVM_PUBLIC_INPUTS_GAS_SETTINGS_MAX_FEES_PER_GAS_ROW_IDX: u32 =
    AVM_PUBLIC_INPUTS_GAS_SETTINGS_TEARDOWN_GAS_LIMITS_ROW_IDX + 1;
pub global AVM_PUBLIC_INPUTS_GAS_SETTINGS_MAX_PRIORITY_FEES_PER_GAS_ROW_IDX: u32 =
    AVM_PUBLIC_INPUTS_GAS_SETTINGS_MAX_FEES_PER_GAS_ROW_IDX + 1;
// Note: each of these gas settings ^, despite being 2 entries each, are split across 2 columns. So we only bump by 1 for each.

// Fee Payer
pub global AVM_PUBLIC_INPUTS_FEE_PAYER_ROW_IDX: u32 =
    AVM_PUBLIC_INPUTS_GAS_SETTINGS_MAX_PRIORITY_FEES_PER_GAS_ROW_IDX + 1;

// Public Setup Call Requests
// Note: PublicCallRequests are a few entries, but are split across multiple columns so that they consume 1 row each.
// So we only bump by 1 (below) to progress.
pub global AVM_PUBLIC_INPUTS_PUBLIC_SETUP_CALL_REQUESTS_ROW_IDX: u32 =
    AVM_PUBLIC_INPUTS_FEE_PAYER_ROW_IDX + 1;
// Public App Logic Call Requests
pub global AVM_PUBLIC_INPUTS_PUBLIC_APP_LOGIC_CALL_REQUESTS_ROW_IDX: u32 =
    AVM_PUBLIC_INPUTS_PUBLIC_SETUP_CALL_REQUESTS_ROW_IDX + MAX_ENQUEUED_CALLS_PER_TX;
// Public Teardown Call Request (just 1)
pub global AVM_PUBLIC_INPUTS_PUBLIC_TEARDOWN_CALL_REQUEST_ROW_IDX: u32 =
    AVM_PUBLIC_INPUTS_PUBLIC_APP_LOGIC_CALL_REQUESTS_ROW_IDX + MAX_ENQUEUED_CALLS_PER_TX;

// Previous Non-Revertible Accumulated Data Array Lengths
// Note: accumulated data array lengths are a few entries, but are split across multiple columns so that they consume 1 row each.
pub global AVM_PUBLIC_INPUTS_PREVIOUS_NON_REVERTIBLE_ACCUMULATED_DATA_ARRAY_LENGTHS_ROW_IDX: u32 =
    AVM_PUBLIC_INPUTS_PUBLIC_TEARDOWN_CALL_REQUEST_ROW_IDX + 1;
// Previous Revertible Accumulated Data Array Lengths
pub global AVM_PUBLIC_INPUTS_PREVIOUS_REVERTIBLE_ACCUMULATED_DATA_ARRAY_LENGTHS_ROW_IDX: u32 =
    AVM_PUBLIC_INPUTS_PREVIOUS_NON_REVERTIBLE_ACCUMULATED_DATA_ARRAY_LENGTHS_ROW_IDX + 1;

// Previous Non-Revertible Accumulated Data
// start alias
pub global AVM_PUBLIC_INPUTS_PREVIOUS_NON_REVERTIBLE_ACCUMULATED_DATA_ROW_IDX: u32 =
    AVM_PUBLIC_INPUTS_PREVIOUS_REVERTIBLE_ACCUMULATED_DATA_ARRAY_LENGTHS_ROW_IDX + 1;
// members
pub global AVM_PUBLIC_INPUTS_PREVIOUS_NON_REVERTIBLE_ACCUMULATED_DATA_NOTE_HASHES_ROW_IDX: u32 =
    AVM_PUBLIC_INPUTS_PREVIOUS_NON_REVERTIBLE_ACCUMULATED_DATA_ROW_IDX;
pub global AVM_PUBLIC_INPUTS_PREVIOUS_NON_REVERTIBLE_ACCUMULATED_DATA_NULLIFIERS_ROW_IDX: u32 = AVM_PUBLIC_INPUTS_PREVIOUS_NON_REVERTIBLE_ACCUMULATED_DATA_NOTE_HASHES_ROW_IDX
    + MAX_NOTE_HASHES_PER_TX;
pub global AVM_PUBLIC_INPUTS_PREVIOUS_NON_REVERTIBLE_ACCUMULATED_DATA_L2_TO_L1_MSGS_ROW_IDX: u32 = AVM_PUBLIC_INPUTS_PREVIOUS_NON_REVERTIBLE_ACCUMULATED_DATA_NULLIFIERS_ROW_IDX
    + MAX_NULLIFIERS_PER_TX;

// Previous Revertible Accumulated Data
// start alias
pub global AVM_PUBLIC_INPUTS_PREVIOUS_REVERTIBLE_ACCUMULATED_DATA_ROW_IDX: u32 = AVM_PUBLIC_INPUTS_PREVIOUS_NON_REVERTIBLE_ACCUMULATED_DATA_L2_TO_L1_MSGS_ROW_IDX
    + MAX_L2_TO_L1_MSGS_PER_TX;
// members
pub global AVM_PUBLIC_INPUTS_PREVIOUS_REVERTIBLE_ACCUMULATED_DATA_NOTE_HASHES_ROW_IDX: u32 =
    AVM_PUBLIC_INPUTS_PREVIOUS_REVERTIBLE_ACCUMULATED_DATA_ROW_IDX;
pub global AVM_PUBLIC_INPUTS_PREVIOUS_REVERTIBLE_ACCUMULATED_DATA_NULLIFIERS_ROW_IDX: u32 = AVM_PUBLIC_INPUTS_PREVIOUS_REVERTIBLE_ACCUMULATED_DATA_NOTE_HASHES_ROW_IDX
    + MAX_NOTE_HASHES_PER_TX;
pub global AVM_PUBLIC_INPUTS_PREVIOUS_REVERTIBLE_ACCUMULATED_DATA_L2_TO_L1_MSGS_ROW_IDX: u32 = AVM_PUBLIC_INPUTS_PREVIOUS_REVERTIBLE_ACCUMULATED_DATA_NULLIFIERS_ROW_IDX
    + MAX_NULLIFIERS_PER_TX;

// End Tree Snapshots
// start alias
pub global AVM_PUBLIC_INPUTS_END_TREE_SNAPSHOTS_ROW_IDX: u32 = AVM_PUBLIC_INPUTS_PREVIOUS_REVERTIBLE_ACCUMULATED_DATA_L2_TO_L1_MSGS_ROW_IDX
    + MAX_L2_TO_L1_MSGS_PER_TX;
// Note: each tree snapshot ^, despite being 2 entries (root, index), is split across 2 columns. So we only bump by 1 (below) to progress.
// members
pub global AVM_PUBLIC_INPUTS_END_TREE_SNAPSHOTS_L1_TO_L2_MESSAGE_TREE_ROW_IDX: u32 =
    AVM_PUBLIC_INPUTS_END_TREE_SNAPSHOTS_ROW_IDX;
pub global AVM_PUBLIC_INPUTS_END_TREE_SNAPSHOTS_NOTE_HASH_TREE_ROW_IDX: u32 =
    AVM_PUBLIC_INPUTS_END_TREE_SNAPSHOTS_L1_TO_L2_MESSAGE_TREE_ROW_IDX + 1;
pub global AVM_PUBLIC_INPUTS_END_TREE_SNAPSHOTS_NULLIFIER_TREE_ROW_IDX: u32 =
    AVM_PUBLIC_INPUTS_END_TREE_SNAPSHOTS_NOTE_HASH_TREE_ROW_IDX + 1;
pub global AVM_PUBLIC_INPUTS_END_TREE_SNAPSHOTS_PUBLIC_DATA_TREE_ROW_IDX: u32 =
    AVM_PUBLIC_INPUTS_END_TREE_SNAPSHOTS_NULLIFIER_TREE_ROW_IDX + 1;

// End Gas Used
pub global AVM_PUBLIC_INPUTS_END_GAS_USED_ROW_IDX: u32 =
    AVM_PUBLIC_INPUTS_END_TREE_SNAPSHOTS_PUBLIC_DATA_TREE_ROW_IDX + 1;

// End Accumulated Data
// start alias
pub global AVM_PUBLIC_INPUTS_END_ACCUMULATED_DATA_ROW_IDX: u32 =
    AVM_PUBLIC_INPUTS_END_GAS_USED_ROW_IDX + 1;
// members
pub global AVM_PUBLIC_INPUTS_END_ACCUMULATED_DATA_NOTE_HASHES_ROW_IDX: u32 =
    AVM_PUBLIC_INPUTS_END_ACCUMULATED_DATA_ROW_IDX;
pub global AVM_PUBLIC_INPUTS_END_ACCUMULATED_DATA_NULLIFIERS_ROW_IDX: u32 =
    AVM_PUBLIC_INPUTS_END_ACCUMULATED_DATA_NOTE_HASHES_ROW_IDX + MAX_NOTE_HASHES_PER_TX;
pub global AVM_PUBLIC_INPUTS_END_ACCUMULATED_DATA_L2_TO_L1_MSGS_ROW_IDX: u32 =
    AVM_PUBLIC_INPUTS_END_ACCUMULATED_DATA_NULLIFIERS_ROW_IDX + MAX_NULLIFIERS_PER_TX;
pub global AVM_PUBLIC_INPUTS_END_ACCUMULATED_DATA_PUBLIC_LOGS_ROW_IDX: u32 =
    AVM_PUBLIC_INPUTS_END_ACCUMULATED_DATA_L2_TO_L1_MSGS_ROW_IDX + MAX_L2_TO_L1_MSGS_PER_TX;
pub global AVM_PUBLIC_INPUTS_END_ACCUMULATED_DATA_PUBLIC_DATA_WRITES_ROW_IDX: u32 = AVM_PUBLIC_INPUTS_END_ACCUMULATED_DATA_PUBLIC_LOGS_ROW_IDX
    + (MAX_PUBLIC_LOGS_PER_TX * PUBLIC_LOG_SIZE_IN_FIELDS);

// End Transaction Fee
pub global AVM_PUBLIC_INPUTS_TRANSACTION_FEE_ROW_IDX: u32 = AVM_PUBLIC_INPUTS_END_ACCUMULATED_DATA_PUBLIC_DATA_WRITES_ROW_IDX
    + MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX;

// End Reverted
pub global AVM_PUBLIC_INPUTS_REVERTED_ROW_IDX: u32 = AVM_PUBLIC_INPUTS_TRANSACTION_FEE_ROW_IDX + 1;

// AVM's max public input column size (aka total number of rows)
pub global AVM_PUBLIC_INPUTS_COLUMNS_MAX_LENGTH: u32 = AVM_PUBLIC_INPUTS_REVERTED_ROW_IDX + 1;
pub global AVM_NUM_PUBLIC_INPUT_COLUMNS: u32 = 4;
// Combined size of the AVM's public input columns concatenated together
pub global AVM_PUBLIC_INPUTS_COLUMNS_COMBINED_LENGTH: u32 =
    AVM_NUM_PUBLIC_INPUT_COLUMNS * AVM_PUBLIC_INPUTS_COLUMNS_MAX_LENGTH;

// `AVM_PROOF_LENGTH_IN_FIELDS` must be updated when AVM circuit changes.
// To determine latest value, hover `COMPUTED_AVM_PROOF_LENGTH_IN_FIELDS`
// in barretenberg/cpp/src/barretenberg/vm/avm/generated/flavor.hpp
// This also changes the length of inputs for the public base rollup circuit, so its Prover.toml must be updated.
// To do so: run: 'AZTEC_GENERATE_TEST_DATA=1 yarn workspace @aztec/prover-client test orchestrator_public_functions' from yarn-project.
pub global AVM_PROOF_LENGTH_IN_FIELDS: u32 = 4154;
pub global AVM_PUBLIC_COLUMN_MAX_SIZE: u32 = 1024;

pub global AVM_V2_PUBLIC_INPUTS_FLATTENED_SIZE: u32 = 1;
// pub global AVM_V2_PROOF_LENGTH_IN_FIELDS: u32 = 5452;
// VK is composed of
// - circuit size encoded as a fr field element (32 bytes)
// - num of inputs encoded as a fr field element (32 bytes)
// - 45 affine elements (curve base field fq) encoded as fr elements takes (45 * 4 * 32 bytes)
// 45 above refers to the constant AvmFlavorVariables::NUM_PRECOMPUTED_ENTITIES
//pub global AVM_V2_VERIFICATION_KEY_LENGTH_IN_FIELDS: u32 = 2 + 45 * 4;

// We introduce padded version for two AVM constants and we will use the padded ones
// until we freeze the number of columns in AVM.
// TODO(#13390): Remove these constants and use the above ones (commented out) in noir/TS.
pub global AVM_V2_PROOF_LENGTH_IN_FIELDS_PADDED: u32 = 20000;
pub global AVM_V2_VERIFICATION_KEY_LENGTH_IN_FIELDS_PADDED: u32 = 1000;

// GAS DEFAULTS
pub global DEFAULT_GAS_LIMIT: u32 = 1_000_000_000;
pub global MAX_L2_GAS_PER_TX_PUBLIC_PORTION: u32 = 6_000_000;
pub global DEFAULT_TEARDOWN_GAS_LIMIT: u32 = MAX_L2_GAS_PER_TX_PUBLIC_PORTION;
pub global DA_BYTES_PER_FIELD: u32 = 32;
pub global DA_GAS_PER_BYTE: u32 = 16;
// pays for preamble information in TX Effects
pub global FIXED_DA_GAS: u32 = 512;
// pays for fixed tx costs like validation, and updating state roots
pub global FIXED_L2_GAS: u32 = 512;
// base cost for a single public call
pub global FIXED_AVM_STARTUP_L2_GAS: u32 = 20_000;

// Some tree insertions incur an additional cost associated with
// the new database entry to be stored by all network participants.
pub global L2_GAS_DISTRIBUTED_STORAGE_PREMIUM: u32 = 1024;

// All AVM opcode gas costs are below except this one because it is needed for Merkle tree related constants.
pub global AVM_POSEIDON2_BASE_L2_GAS: u32 = 36;

// Kernel and/or rollup circuits perform a hash per tree-level for insertions and reads.
// All network participants need to perform tree insertions.
// Not _all_ network participants need to perform membership checks, so they're cheaper.
pub global L2_GAS_PER_READ_MERKLE_HASH: u32 = AVM_POSEIDON2_BASE_L2_GAS;
pub global L2_GAS_PER_WRITE_MERKLE_HASH: u32 = AVM_POSEIDON2_BASE_L2_GAS;

// Gas for tree insertions and associated storage
// For public data tree update, we account for the worst case scenario which is a new insertion
// takes 4 Merkle path checks while an update would need 3 Merkle path checks.
pub global L2_GAS_PER_PUBLIC_DATA_UPDATE: u32 = L2_GAS_DISTRIBUTED_STORAGE_PREMIUM
    + (4 * PUBLIC_DATA_TREE_HEIGHT * L2_GAS_PER_WRITE_MERKLE_HASH);
pub global L2_GAS_PER_NOTE_HASH: u32 =
    L2_GAS_DISTRIBUTED_STORAGE_PREMIUM + (2 * NOTE_HASH_TREE_HEIGHT * L2_GAS_PER_WRITE_MERKLE_HASH);
// 2x because insertion into indexed tree requires a low-leaf membership check and a standard insertion
pub global L2_GAS_PER_NULLIFIER: u32 =
    L2_GAS_DISTRIBUTED_STORAGE_PREMIUM + (3 * NULLIFIER_TREE_HEIGHT * L2_GAS_PER_WRITE_MERKLE_HASH);

// Gas for tree read requests
pub global L2_GAS_PER_PUBLIC_DATA_READ: u32 = PUBLIC_DATA_TREE_HEIGHT * L2_GAS_PER_READ_MERKLE_HASH;
pub global L2_GAS_PER_NOTE_HASH_READ_REQUEST: u32 =
    NOTE_HASH_TREE_HEIGHT * L2_GAS_PER_READ_MERKLE_HASH;
// 2x because non-membership checks are really 2 membership checks
pub global L2_GAS_PER_NULLIFIER_READ_REQUEST: u32 =
    NULLIFIER_TREE_HEIGHT * L2_GAS_PER_READ_MERKLE_HASH;
pub global L2_GAS_PER_L1_TO_L2_MSG_READ_REQUEST: u32 =
    L1_TO_L2_MSG_TREE_HEIGHT * L2_GAS_PER_READ_MERKLE_HASH;

// Gas for hashing and validating logs
pub global L2_GAS_PER_LOG_BYTE: u32 = 4;
// Zero gas because we don't have to hash and validate the private logs
pub global L2_GAS_PER_PRIVATE_LOG: u32 = 0;
pub global L2_GAS_PER_CONTRACT_CLASS_LOG: u32 = 0;

// Gas for writing message to L1 portal
pub global L2_GAS_PER_L2_TO_L1_MSG: u32 = 200;

/**
 * GAS COSTS FOR AVM OPCODES
 * They are used in TS and C++
 */

// Exception for AVM_POSEIDON2_BASE_L2_GAS which needed to be defined above.

// Base L2 GAS
// Note: magic numbers here are derived from each op's AVM circuit trace area
pub global AVM_ADD_BASE_L2_GAS: u32 = 27;
pub global AVM_SUB_BASE_L2_GAS: u32 = 27;
pub global AVM_MUL_BASE_L2_GAS: u32 = 27;
pub global AVM_DIV_BASE_L2_GAS: u32 = 27;
pub global AVM_FDIV_BASE_L2_GAS: u32 = 27;
pub global AVM_EQ_BASE_L2_GAS: u32 = 27;
pub global AVM_LT_BASE_L2_GAS: u32 = 30;
pub global AVM_LTE_BASE_L2_GAS: u32 = 30;
pub global AVM_AND_BASE_L2_GAS: u32 = 51;
pub global AVM_OR_BASE_L2_GAS: u32 = 51;
pub global AVM_XOR_BASE_L2_GAS: u32 = 51;
pub global AVM_NOT_BASE_L2_GAS: u32 = 18;
pub global AVM_SHL_BASE_L2_GAS: u32 = 27;
pub global AVM_SHR_BASE_L2_GAS: u32 = 27;
pub global AVM_CAST_BASE_L2_GAS: u32 = 18;
pub global AVM_GETENVVAR_BASE_L2_GAS: u16 = 9;
pub global AVM_CALLDATACOPY_BASE_L2_GAS: u32 = 24;
pub global AVM_SUCCESSCOPY_BASE_L2_GAS: u32 = 24;
pub global AVM_RETURNDATASIZE_BASE_L2_GAS: u32 = 9;
pub global AVM_RETURNDATACOPY_BASE_L2_GAS: u32 = 24;
pub global AVM_JUMP_BASE_L2_GAS: u32 = 3;
pub global AVM_JUMPI_BASE_L2_GAS: u32 = 9;
pub global AVM_INTERNALCALL_BASE_L2_GAS: u32 = 3;
pub global AVM_INTERNALRETURN_BASE_L2_GAS: u32 = 3;
pub global AVM_SET_BASE_L2_GAS: u32 = 9;
pub global AVM_MOV_BASE_L2_GAS: u32 = 18;
pub global AVM_SLOAD_BASE_L2_GAS: u32 = 18 + L2_GAS_PER_PUBLIC_DATA_READ;
pub global AVM_SSTORE_BASE_L2_GAS: u32 = 18 + L2_GAS_PER_PUBLIC_DATA_UPDATE;
pub global AVM_NOTEHASHEXISTS_BASE_L2_GAS: u32 = 27 + L2_GAS_PER_NOTE_HASH_READ_REQUEST;
pub global AVM_EMITNOTEHASH_BASE_L2_GAS: u32 = 9 + L2_GAS_PER_NOTE_HASH;
pub global AVM_NULLIFIEREXISTS_BASE_L2_GAS: u32 = 27 + L2_GAS_PER_NULLIFIER_READ_REQUEST;
pub global AVM_EMITNULLIFIER_BASE_L2_GAS: u32 = 9 + L2_GAS_PER_NULLIFIER;
pub global AVM_L1TOL2MSGEXISTS_BASE_L2_GAS: u32 = 27 + L2_GAS_PER_L1_TO_L2_MSG_READ_REQUEST;
pub global AVM_GETCONTRACTINSTANCE_BASE_L2_GAS: u32 = 27 + L2_GAS_PER_NULLIFIER_READ_REQUEST; // does a nullifier check
pub global AVM_EMITUNENCRYPTEDLOG_BASE_L2_GAS: u32 = 15;
pub global AVM_SENDL2TOL1MSG_BASE_L2_GAS: u32 = 18 + L2_GAS_PER_L2_TO_L1_MSG;
// On CALL, AVM performs nullifier checks for contract address & contract class ID
pub global AVM_CALL_BASE_L2_GAS: u32 = 45 + (2 * L2_GAS_PER_NULLIFIER_READ_REQUEST);
pub global AVM_STATICCALL_BASE_L2_GAS: u32 = 45 + (2 * L2_GAS_PER_NULLIFIER_READ_REQUEST);
pub global AVM_RETURN_BASE_L2_GAS: u32 = 15;
pub global AVM_REVERT_BASE_L2_GAS: u32 = 15;
pub global AVM_DEBUGLOG_BASE_L2_GAS: u32 = 21;
// For POSEIDON2 see above (constant needed to be set before L2_GAS_PER_READ_MERKLE_HASH)
pub global AVM_SHA256COMPRESSION_BASE_L2_GAS: u32 = 114;
pub global AVM_KECCAKF1600_BASE_L2_GAS: u32 = 162;
pub global AVM_ECADD_BASE_L2_GAS: u32 = 69;
pub global AVM_MSM_BASE_L2_GAS: u32 = 36;
pub global AVM_TORADIXBE_BASE_L2_GAS: u32 = 24;

// Dynamic L2 GAS
pub global AVM_CALLDATACOPY_DYN_L2_GAS: u32 = 3;
pub global AVM_RETURNDATACOPY_DYN_L2_GAS: u32 = 3;
// a single increment here corresponds to an entire additional field (hence x32 bytes per field)
pub global AVM_EMITUNENCRYPTEDLOG_DYN_L2_GAS: u32 = 3 + (DA_BYTES_PER_FIELD * L2_GAS_PER_LOG_BYTE);
pub global AVM_CALL_DYN_L2_GAS: u32 = 4;
pub global AVM_STATICCALL_DYN_L2_GAS: u32 = 4;
pub global AVM_RETURN_DYN_L2_GAS: u32 = 3;
pub global AVM_REVERT_DYN_L2_GAS: u32 = 3;
pub global AVM_MSM_DYN_L2_GAS: u32 = 6;
pub global AVM_TORADIXBE_DYN_L2_GAS: u32 = 3;

// Base DA Gas
pub global AVM_SSTORE_BASE_DA_GAS: u32 = DA_BYTES_PER_FIELD * DA_GAS_PER_BYTE;
pub global AVM_EMITNOTEHASH_BASE_DA_GAS: u32 = DA_BYTES_PER_FIELD * DA_GAS_PER_BYTE;
pub global AVM_EMITNULLIFIER_BASE_DA_GAS: u32 = DA_BYTES_PER_FIELD * DA_GAS_PER_BYTE;
pub global AVM_SENDL2TOL1MSG_BASE_DA_GAS: u32 = DA_BYTES_PER_FIELD * DA_GAS_PER_BYTE;

// Dynamic DA Gas
// a single increment here corresponds to an entire additional field (hence x32 bytes per field)
pub global AVM_EMITUNENCRYPTEDLOG_DYN_DA_GAS: u32 = DA_BYTES_PER_FIELD * DA_GAS_PER_BYTE;

// BLOB PREFIXES
// Used when decoding blobs of tightly packed effects
pub global TX_START_PREFIX: Field = 0x74785f7374617274; // = "tx_start".to_field() in nr
pub global REVERT_CODE_PREFIX: u8 = 1;
pub global TX_FEE_PREFIX: u8 = 2;
pub global NOTES_PREFIX: u8 = 3;
pub global NULLIFIERS_PREFIX: u8 = 4;
pub global L2_L1_MSGS_PREFIX: u8 = 5;
pub global PUBLIC_DATA_UPDATE_REQUESTS_PREFIX: u8 = 6;
pub global PRIVATE_LOGS_PREFIX: u8 = 7;
pub global PUBLIC_LOGS_PREFIX: u8 = 8;
pub global CONTRACT_CLASS_LOGS_PREFIX: u8 = 9;

// Constants related to proof type of a recursive proof verification.
// Keep following constants in sync with the enum acir_format::PROOF_TYPE in recursion_constraint.hpp
pub global PROOF_TYPE_PLONK: u32 = 0;
pub global PROOF_TYPE_HONK: u32 = 1;
pub global PROOF_TYPE_OINK: u32 = 2;
pub global PROOF_TYPE_PG: u32 = 3;
pub global PROOF_TYPE_AVM: u32 = 4;
pub global PROOF_TYPE_ROLLUP_HONK: u32 = 5;
pub global PROOF_TYPE_ROOT_ROLLUP_HONK: u32 = 6;

// AVM - misc constants
pub global AVM_PC_SIZE_IN_BITS: u8 = 32;
pub global TWO_POW_64: Field = 18446744073709551616;
pub global AVM_MAX_OPERANDS: u8 = 8;

pub global BLOCK_NUMBER_BIT_SIZE: u32 = 32;

// 24 hours with a slot duration of 24 seconds
pub global DEFAULT_UPDATE_DELAY: u32 = 3600;
// 10 minutes with a slot duration of 24 seconds
pub global MINIMUM_UPDATE_DELAY: u32 = 25;

// Updates hold contract class id, which fit in 1 field.
pub global UPDATES_VALUE_SIZE: u32 = 1;
pub global UPDATES_SHARED_MUTABLE_VALUES_LEN: u32 = 2 * UPDATES_VALUE_SIZE + 1;

// These bit sizes are used in PIL. If need change, consult with the AVM team.
pub global UPDATES_SHARED_MUTABLE_SDC_DELAY_BIT_SIZE: u32 = 32;
pub global UPDATES_SHARED_MUTABLE_SDC_IS_SOME_BIT_SIZE: u32 = 8;
pub global UPDATES_SHARED_MUTABLE_SDC_OPTION_DELAY_BIT_SIZE: u32 =
    UPDATES_SHARED_MUTABLE_SDC_DELAY_BIT_SIZE + UPDATES_SHARED_MUTABLE_SDC_IS_SOME_BIT_SIZE;
pub global UPDATES_SHARED_MUTABLE_METADATA_BIT_SIZE: u32 =
    UPDATES_SHARED_MUTABLE_SDC_OPTION_DELAY_BIT_SIZE * 2 + BLOCK_NUMBER_BIT_SIZE * 2;

pub global GRUMPKIN_ONE_X: Field = 1;
pub global GRUMPKIN_ONE_Y: Field = 17631683881184975370165255887551781615748388533673675138860;

mod test {
    use crate::constants::{
        MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
    };

    #[test]
    unconstrained fn test_constants() {
        assert(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL <= MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX);
    }
}
