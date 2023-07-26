use crate::constants_gen::{
    RETURN_VALUES_LENGTH,
    MAX_READ_REQUESTS_PER_CALL,
    MAX_NEW_COMMITMENTS_PER_CALL,
    MAX_NEW_NULLIFIERS_PER_CALL,
    MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL,
    MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
    MAX_NEW_L2_TO_L1_MSGS_PER_CALL,
    NUM_FIELDS_PER_SHA256,
    MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL,
    MAX_PUBLIC_DATA_READS_PER_CALL,
    GENERATOR_INDEX__FUNCTION_ARGS,
    COMMITMENT_TREES_ROOTS_LENGTH,
    CONTRACT_DEPLOYMENT_DATA_LENGTH,
    CALL_CONTEXT_LENGTH,
    PRIVATE_CIRCUIT_PUBLIC_INPUTS_LENGTH,
    CONTRACT_STORAGE_UPDATE_REQUEST_LENGTH,
    CONTRACT_STORAGE_READ_LENGTH,
    PUBLIC_CIRCUIT_PUBLIC_INPUTS_LENGTH,
    PUBLIC_CIRCUIT_PUBLIC_INPUTS_HASH_INPUT_LENGTH,
    GENERATOR_INDEX__PUBLIC_CIRCUIT_PUBLIC_INPUTS,
    GENERATOR_INDEX__FUNCTION_DATA,
    GENERATOR_INDEX__PUBLIC_DATA_READ,
    GENERATOR_INDEX__PUBLIC_DATA_UPDATE_REQUEST,
    GENERATOR_INDEX__CALL_CONTEXT,
};

use crate::oracle::debug_log;
use crate::types::vec::BoundedVec;
use crate::types::point::Point;

// PrivateContextInputs are expected to be provided to each private function
struct PrivateContextInputs {
    call_context : CallContext,
    roots: CommitmentTreesRoots,

    contract_deployment_data: ContractDeploymentData,

    private_global_variables: PrivateGlobalVariables,
}

// PublicContextInputs are expected to be provided to each public function
struct PublicContextInputs {
    call_context: CallContext,
    roots: CommitmentTreesRoots,

    public_global_variables: PublicGlobalVariables,
}

struct CallContext {
    msg_sender : Field,
    storage_contract_address : Field,
    portal_contract_address : Field,

    is_delegate_call : bool,
    is_static_call : bool,
    is_contract_deployment: bool,
}

impl CallContext {
    fn serialize(self) -> [Field; CALL_CONTEXT_LENGTH] {
        [
            self.msg_sender,
            self.storage_contract_address,
            self.portal_contract_address,
            self.is_delegate_call as Field,
            self.is_static_call as Field,
            self.is_contract_deployment as Field,
        ]
    }

    fn hash(self) -> Field {
        dep::std::hash::pedersen_with_separator(self.serialize(), GENERATOR_INDEX__CALL_CONTEXT)[0]
    }
}

struct CommitmentTreesRoots {
    private_data_tree_root : Field,
    nullifier_tree_root : Field,
    contract_tree_root : Field,
    l1_to_l2_messages_tree_root : Field,
    blocks_tree_root: Field,
}

impl CommitmentTreesRoots {
    // NOTE: this order must match the order in `private_circuit_public_inputs.hpp`
    fn serialize(self) -> [Field; COMMITMENT_TREES_ROOTS_LENGTH] {
        [
            self.private_data_tree_root,
            self.nullifier_tree_root,
            self.contract_tree_root,
            self.l1_to_l2_messages_tree_root,
            self.blocks_tree_root,
        ]
    }
}

fn empty_commitment_trees_roots() -> CommitmentTreesRoots {
    CommitmentTreesRoots{ private_data_tree_root: 0, nullifier_tree_root: 0, contract_tree_root: 0, l1_to_l2_messages_tree_root: 0, blocks_tree_root: 0 }
}

struct ContractDeploymentData {
    deployer_public_key: Point,
    constructor_vk_hash : Field,
    function_tree_root : Field,
    contract_address_salt : Field,
    portal_contract_address : Field,
}

impl ContractDeploymentData {
    fn serialize(self) -> [Field; CONTRACT_DEPLOYMENT_DATA_LENGTH] {
        [
            self.deployer_public_key.x,
            self.deployer_public_key.y,
            self.constructor_vk_hash,
            self.function_tree_root,
            self.contract_address_salt,
            self.portal_contract_address,
        ]
    }
}

struct PrivateGlobalVariables {
    chain_id: Field,
    version: Field,
}

impl PrivateGlobalVariables {
    fn serialize(self) -> [Field; 2] {
        [self.chain_id, self.version]
    }
}

struct PublicGlobalVariables {
    chain_id: Field,
    version: Field,
    block_number: Field,
    timestamp: Field,
}

impl PublicGlobalVariables {
    fn serialize(self) -> [Field; 4] {
        [self.chain_id, self.version, self.block_number, self.timestamp]
    }
}

struct FunctionData {
    function_selector: Field,
    is_private: bool,
    is_constructor: bool,
}

impl FunctionData {
    fn hash(self) -> Field {
        dep::std::hash::pedersen_with_separator([
            self.function_selector,
            self.is_private as Field,
            self.is_constructor as Field,
        ], GENERATOR_INDEX__FUNCTION_DATA)[0]
    }
}

struct PrivateCircuitPublicInputs {
    call_context: CallContext,
    args_hash: Field,
    return_values: [Field; RETURN_VALUES_LENGTH],
    read_requests: [Field; crate::abi::MAX_READ_REQUESTS_PER_CALL],
    new_commitments: [Field; MAX_NEW_COMMITMENTS_PER_CALL],
    new_nullifiers: [Field; MAX_NEW_NULLIFIERS_PER_CALL],
    nullified_commitments: [Field; MAX_NEW_NULLIFIERS_PER_CALL],
    private_call_stack: [Field; MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL],
    public_call_stack: [Field; MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL],
    new_l2_to_l1_msgs: [Field; MAX_NEW_L2_TO_L1_MSGS_PER_CALL],
    // Explore introducing a new type like uint256 (similar to Point), so it's more explicit that
    // we're talking about a single number backed by two field elements.
    encrypted_logs_hash: [Field; NUM_FIELDS_PER_SHA256],
    unencrypted_logs_hash: [Field; NUM_FIELDS_PER_SHA256],
    encrypted_log_preimages_length: Field,
    unencrypted_log_preimages_length: Field,
    commitment_trees_roots: CommitmentTreesRoots,
    contract_deployment_data: ContractDeploymentData,
    chain_id: Field,
    version: Field,
}

impl PrivateCircuitPublicInputs {
    // TODO hash_index
    fn hash(self) -> Field {
        dep::std::hash::pedersen(self.serialize())[0]
    }

    fn serialize(self) -> [Field; PRIVATE_CIRCUIT_PUBLIC_INPUTS_LENGTH] {
        let mut fields: BoundedVec<Field, PRIVATE_CIRCUIT_PUBLIC_INPUTS_LENGTH> = BoundedVec::new(0); 
        fields = fields.push_array(self.call_context.serialize());
        fields = fields.push(self.args_hash);
        fields = fields.push_array(self.return_values);
        fields = fields.push_array(self.read_requests);
        fields = fields.push_array(self.new_commitments);
        fields = fields.push_array(self.new_nullifiers);
        fields = fields.push_array(self.nullified_commitments);
        fields = fields.push_array(self.private_call_stack);
        fields = fields.push_array(self.public_call_stack);
        fields = fields.push_array(self.new_l2_to_l1_msgs);
        fields = fields.push_array(self.encrypted_logs_hash);
        fields = fields.push_array(self.unencrypted_logs_hash);
        fields = fields.push(self.encrypted_log_preimages_length);
        fields = fields.push(self.unencrypted_log_preimages_length);
        fields = fields.push_array(self.commitment_trees_roots.serialize());
        fields = fields.push_array(self.contract_deployment_data.serialize());
        fields = fields.push(self.chain_id);
        fields = fields.push(self.version);
        fields.storage
    }
}

struct ContractStorageRead {
    storage_slot: Field,
    value: Field,
}

fn empty_contract_storage_read() -> ContractStorageRead {
    ContractStorageRead { storage_slot: 0, value: 0 }
}

impl ContractStorageRead {
    fn serialize(self) -> [Field; CONTRACT_STORAGE_READ_LENGTH] {
        [self.storage_slot, self.value]
    }

    fn hash(self) -> Field {
        dep::std::hash::pedersen_with_separator(self.serialize(), GENERATOR_INDEX__PUBLIC_DATA_READ)[0]
    }
}

struct ContractStorageUpdateRequest {
    storage_slot: Field,
    old_value: Field,
    new_value: Field,
}

impl ContractStorageUpdateRequest {
    fn serialize(self) -> [Field; CONTRACT_STORAGE_UPDATE_REQUEST_LENGTH] {
        [self.storage_slot, self.old_value, self.new_value]
    }

    fn hash(self) -> Field {
        dep::std::hash::pedersen_with_separator(self.serialize(), GENERATOR_INDEX__PUBLIC_DATA_UPDATE_REQUEST)[0]
    }
}

fn empty_contract_storage_update_request() -> ContractStorageUpdateRequest {
    ContractStorageUpdateRequest { storage_slot: 0, old_value: 0, new_value: 0 }
}


struct PublicCircuitPublicInputs {
    call_context: CallContext,
    args_hash: Field,
    return_values: [Field; RETURN_VALUES_LENGTH],
    contract_storage_update_requests: [ContractStorageUpdateRequest; MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL],
    contract_storage_read: [ContractStorageRead; MAX_PUBLIC_DATA_READS_PER_CALL],
    public_call_stack: [Field; MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL],
    new_commitments: [Field; MAX_NEW_COMMITMENTS_PER_CALL],
    new_nullifiers: [Field; crate::abi::MAX_NEW_NULLIFIERS_PER_CALL],
    new_l2_to_l1_msgs: [Field; crate::abi::MAX_NEW_L2_TO_L1_MSGS_PER_CALL],
    unencrypted_logs_hash: [Field; NUM_FIELDS_PER_SHA256],
    unencrypted_log_preimages_length: Field,
    commitment_trees_roots: CommitmentTreesRoots, // TODO: This is not present in cpp or ts, do we need to include it?
    historic_public_data_tree_root: Field,
    prover_address: Field,
}

impl PublicCircuitPublicInputs {
    
    fn hash(self) -> Field {
        let mut inputs: BoundedVec<Field, PUBLIC_CIRCUIT_PUBLIC_INPUTS_HASH_INPUT_LENGTH> = BoundedVec::new(0);
        inputs = inputs.push(self.call_context.hash());
        inputs = inputs.push(self.args_hash);
        inputs = inputs.push_array(self.return_values);
        for i in 0..MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL {
            inputs = inputs.push(self.contract_storage_update_requests[i].hash());
        }
        for i in 0..MAX_PUBLIC_DATA_READS_PER_CALL {
            inputs = inputs.push(self.contract_storage_read[i].hash());
        }
        inputs = inputs.push_array(self.public_call_stack);
        inputs = inputs.push_array(self.new_commitments);
        inputs = inputs.push_array(self.new_nullifiers);
        inputs = inputs.push_array(self.new_l2_to_l1_msgs);

        // We do not include commitment_trees_roots since it's not in the cpp hash
        // inputs = inputs.push(self.commitment_trees_roots.hash());

        inputs = inputs.push_array(self.unencrypted_logs_hash);
        inputs = inputs.push(self.unencrypted_log_preimages_length);
        inputs = inputs.push(self.historic_public_data_tree_root);
        inputs = inputs.push(self.prover_address);

        dep::std::hash::pedersen_with_separator(inputs.storage, GENERATOR_INDEX__PUBLIC_CIRCUIT_PUBLIC_INPUTS)[0]
    }

    fn serialize(self) -> [Field; PUBLIC_CIRCUIT_PUBLIC_INPUTS_LENGTH] {
        let mut fields: BoundedVec<Field, PUBLIC_CIRCUIT_PUBLIC_INPUTS_LENGTH> = BoundedVec::new(0); 
        fields = fields.push_array(self.call_context.serialize());   
        fields = fields.push(self.args_hash);
        fields = fields.push_array(self.return_values);
        for i in 0..MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL {
            fields = fields.push_array(self.contract_storage_update_requests[i].serialize());
        }
        for i in 0..MAX_PUBLIC_DATA_READS_PER_CALL {
            fields = fields.push_array(self.contract_storage_read[i].serialize());
        }
        fields = fields.push_array(self.public_call_stack);
        fields = fields.push_array(self.new_commitments);
        fields = fields.push_array(self.new_nullifiers);
        fields = fields.push_array(self.new_l2_to_l1_msgs);
        fields = fields.push_array(self.unencrypted_logs_hash);
        fields = fields.push(self.unencrypted_log_preimages_length);
        fields = fields.push_array(self.commitment_trees_roots.serialize());
        fields = fields.push(self.historic_public_data_tree_root);
        fields = fields.push(self.prover_address);
        fields.storage
    }
}

global ARGS_HASH_CHUNK_LENGTH: u32 = 32;
global ARGS_HASH_CHUNK_COUNT: u32 = 16;

fn hash_args<N>(args: [Field; N]) -> Field {
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
            chunk_hash = dep::std::hash::pedersen_with_separator(chunk_args, GENERATOR_INDEX__FUNCTION_ARGS)[0];
        }
        chunks_hashes[i] = chunk_hash;
    }
    dep::std::hash::pedersen_with_separator(chunks_hashes, GENERATOR_INDEX__FUNCTION_ARGS)[0]
}
