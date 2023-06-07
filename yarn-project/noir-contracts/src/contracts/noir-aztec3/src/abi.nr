global MAX_ARGS: comptime Field = 8;
global MAX_RETURN_VALUES: comptime Field = 4;
global MAX_NEW_COMMITMENTS: comptime Field = 4;
global MAX_NEW_NULLIFIERS: comptime Field = 4;
global MAX_PRIVATE_CALL_STACK : comptime Field = 4;
global MAX_PUBLIC_CALL_STACK: comptime Field = 4;
global MAX_L1_MSG_STACK : comptime Field = 2;
// global LOGS_HASH_SIZE: comptime Field = 2; // Stored in  2 fields in order to fit in all of the 256 bits of sha256 hash.
global PUBLIC_INPUTS_LENGTH : comptime Field = 37;
global CONTRACT_STORAGE_UPDATE_REQUESTS_LENGTH : comptime Field = 4;
global CONTRACT_STORAGE_READS_LENGTH : comptime Field = 4;

use crate::types::vec::BoundedVec;

// Inputs are expected to be provided to each contract function
struct Inputs {
    call_context : CallContext,

    old_private_data_tree_root : Field,
    old_nullifier_tree_root : Field,
    old_contract_tree_root : Field,
    old_l1_to_l2_messages_tree_root : Field,

    contract_deployment_data: ContractDeploymentData,
}

global CALL_CONTEXT_SIZE: comptime Field = 6;

struct CallContext {
    msg_sender : Field,
    storage_contract_address : Field,
    portal_contract_address : Field,

    is_delegate_call : bool,
    is_static_call : bool,
    is_contract_deployment: bool,
}

impl CallContext {
    fn serialize(self) -> [Field; CALL_CONTEXT_SIZE] {
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
        // TODO hash_index
        dep::std::hash::pedersen(self.serialize())[0]
    }
}

struct ContractDeploymentData {
    constructor_vk_hash : Field,
    function_tree_root : Field,
    contract_address_salt : Field,
    portal_contract_address : Field,
}

impl ContractDeploymentData {
    fn serialize(self) -> [Field; 4] {
        [
            self.constructor_vk_hash,
            self.function_tree_root,
            self.contract_address_salt,
            self.portal_contract_address,
        ]
    }
}

global FUNCTION_DATA_SIZE: comptime Field = 3;

struct FunctionData {
    function_selector: Field,
    is_private: bool,
    is_constructor: bool,
}

impl FunctionData {
    fn hash(self) -> Field {
        dep::std::hash::pedersen([
            // TODO hash_index
            self.function_selector,
            self.is_private as Field,
            self.is_constructor as Field,
        ])[0]
    }
}

struct PrivateCircuitPublicInputs {
    call_context: CallContext,
    args_hash: Field,
    return_values: [Field; crate::abi::MAX_RETURN_VALUES],
    new_commitments: [Field; crate::abi::MAX_NEW_COMMITMENTS],
    new_nullifiers: [Field; crate::abi::MAX_NEW_NULLIFIERS],
    private_call_stack: [Field; crate::abi::MAX_PRIVATE_CALL_STACK],
    public_call_stack: [Field; crate::abi::MAX_PUBLIC_CALL_STACK],
    new_l2_to_l1_msgs: [Field; crate::abi::MAX_L1_MSG_STACK],
    // TODO #588, relevant issue: https://github.com/AztecProtocol/aztec-packages/issues/588
    // When uncommenting, explore introducing a new type like uint256 (similar to Point), so it's more explicit that
    // we're talking about a single number backed by two field elements.
    // encrypted_logs_hash: [Field; crate::abi::LOGS_HASH_SIZE],
    // unencrypted_logs_hash: [Field; crate::abi::LOGS_HASH_SIZE],
    // encrypted_log_preimages_length: Field,
    // unencrypted_log_preimages_length: Field,
    historic_private_data_tree_root: Field,
    historic_private_nullifier_tree_root: Field,
    historic_contract_tree_root: Field,
    historic_l1_to_l2_messages_tree_root: Field,
    contract_deployment_data: ContractDeploymentData,
}

impl PrivateCircuitPublicInputs {
    // TODO hash_index
    fn hash(self) -> Field {
        dep::std::hash::pedersen(self.serialize())[0]
    }

    fn serialize(self) -> [Field; PUBLIC_INPUTS_LENGTH] {
        let mut fields: BoundedVec<Field, PUBLIC_INPUTS_LENGTH> = BoundedVec::new(0); 
        fields = fields.push_array(self.call_context.serialize());   
        fields = fields.push(self.args_hash);
        fields = fields.push_array(self.return_values);
        fields = fields.push_array(self.new_commitments);
        fields = fields.push_array(self.new_nullifiers);
        fields = fields.push_array(self.private_call_stack);
        fields = fields.push_array(self.public_call_stack);
        fields = fields.push_array(self.new_l2_to_l1_msgs);
        // TODO #588, relevant issue: https://github.com/AztecProtocol/aztec-packages/issues/588
        // fields = fields.push_array(self.encrypted_logs_hash);
        // fields = fields.push_array(self.unencrypted_logs_hash);
        // fields = fields.push(self.encrypted_log_preimages_length);
        // fields = fields.push(self.unencrypted_log_preimages_length);
        fields = fields.push(self.historic_private_data_tree_root);
        fields = fields.push(self.historic_private_nullifier_tree_root);
        fields = fields.push(self.historic_contract_tree_root);
        fields = fields.push(self.historic_l1_to_l2_messages_tree_root);
        fields = fields.push_array(self.contract_deployment_data.serialize());
        fields.storage
    }
}

global CONTRACT_STORAGE_READ_SIZE: comptime Field = 2;

struct ContractStorageRead {
    storage_slot: Field,
    value: Field,
}

fn empty_contract_storage_read() -> ContractStorageRead {
    ContractStorageRead { storage_slot: 0, value: 0 }
}

impl ContractStorageRead {
    fn serialize(self) -> [Field; CONTRACT_STORAGE_READ_SIZE] {
        [self.storage_slot, self.value]
    }
}

global CONTRACT_STORAGE_UPDATE_REQUEST_SIZE: comptime Field = 3;

struct ContractStorageUpdateRequest {
    storage_slot: Field,
    old_value: Field,
    new_value: Field,
}

impl ContractStorageUpdateRequest {
    fn serialize(self) -> [Field; CONTRACT_STORAGE_UPDATE_REQUEST_SIZE] {
        [self.storage_slot, self.old_value, self.new_value]
    }
}

fn empty_contract_storage_update_request() -> ContractStorageUpdateRequest {
    ContractStorageUpdateRequest { storage_slot: 0, old_value: 0, new_value: 0 }
}

// CALL_CONTEXT_SIZE
// + ARGS_HASH 
// + MAX_RETURN_VALUES 
// + CONTRACT_STORAGE_UPDATE_REQUESTS_LENGTH * CONTRACT_STORAGE_UPDATE_REQUEST_SIZE
// + CONTRACT_STORAGE_READS_LENGTH * CONTRACT_STORAGE_READ_SIZE
// + MAX_PUBLIC_CALL_STACK
// + MAX_L1_MSG_STACK
// + 2;
// = 6 + 1 + 4 + 4 * 3 + 4 * 2 + 4 + 2 + 2
global PUBLIC_CIRCUIT_PUBLIC_INPUTS_SIZE: comptime Field = 39;

struct PublicCircuitPublicInputs {
    call_context: CallContext,
    args_hash: Field,
    return_values: [Field; crate::abi::MAX_RETURN_VALUES],
    contract_storage_update_requests: [ContractStorageUpdateRequest; crate::abi::CONTRACT_STORAGE_UPDATE_REQUESTS_LENGTH],
    contract_storage_read: [ContractStorageRead; crate::abi::CONTRACT_STORAGE_READS_LENGTH],
    public_call_stack: [Field; crate::abi::MAX_PUBLIC_CALL_STACK],
    new_l2_to_l1_msgs: [Field; crate::abi::MAX_L1_MSG_STACK],
    historic_public_data_tree_root: Field,
    prover_address: Field,
}

impl PublicCircuitPublicInputs {
    // TODO hash_index
    fn hash(self) -> Field {
        dep::std::hash::pedersen(self.serialize())[0]
    }

    fn serialize(self) -> [Field; PUBLIC_CIRCUIT_PUBLIC_INPUTS_SIZE] {
        let mut fields: BoundedVec<Field, PUBLIC_CIRCUIT_PUBLIC_INPUTS_SIZE> = BoundedVec::new(0); 
        fields = fields.push_array(self.call_context.serialize());   
        fields = fields.push(self.args_hash);
        fields = fields.push_array(self.return_values);
        for i in 0..crate::abi::CONTRACT_STORAGE_UPDATE_REQUESTS_LENGTH {
            fields = fields.push_array(self.contract_storage_update_requests[i].serialize());
        }
        for i in 0..crate::abi::CONTRACT_STORAGE_READS_LENGTH {
            fields = fields.push_array(self.contract_storage_read[i].serialize());
        }
        fields = fields.push_array(self.public_call_stack);
        fields = fields.push_array(self.new_l2_to_l1_msgs);
        fields = fields.push(self.historic_public_data_tree_root);
        fields = fields.push(self.prover_address);
        fields.storage
    }
}
