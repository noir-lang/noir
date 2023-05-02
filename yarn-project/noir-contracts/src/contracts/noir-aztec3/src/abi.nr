global MAX_ARGS: comptime Field = 8;
global MAX_RETURN_VALUES: comptime Field = 4;
global MAX_LOGS: comptime Field = 4;
global MAX_NEW_COMMITMENTS: comptime Field = 4;
global MAX_NEW_NULLIFIERS: comptime Field = 4;
global MAX_PRIVATE_CALL_STACK : comptime Field = 4;
global MAX_PUBLIC_CALL_STACK: comptime Field = 4;
global MAX_L1_MSG_STACK : comptime Field = 2;
global PUBLIC_INPUTS_LENGTH : comptime Field = 47;

use crate::types::vec::BoundedVec;

// Inputs are expected to be provided to each contract function
struct Inputs {
    call_context : CallContext,

    old_private_data_tree_root : Field,
    old_nullifier_tree_root : Field,
    old_contract_tree_root : Field,

    contract_deployment_data: ContractDeploymentData,
    args: [Field; MAX_ARGS],
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
    fn serialize(self) -> [Field; 6] {
        [
            self.msg_sender,
            self.storage_contract_address,
            self.portal_contract_address,
            self.is_delegate_call as Field,
            self.is_static_call as Field,
            self.is_contract_deployment as Field,
        ]
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

struct PrivateCircuitPublicInputs {
    call_context: CallContext,
    args: [Field; crate::abi::MAX_ARGS],
    return_values: [Field; crate::abi::MAX_RETURN_VALUES],
    emitted_events: [Field; crate::abi::MAX_LOGS],
    new_commitments: [Field; crate::abi::MAX_NEW_COMMITMENTS],
    new_nullifiers: [Field; crate::abi::MAX_NEW_NULLIFIERS],
    private_call_stack: [Field; crate::abi::MAX_PRIVATE_CALL_STACK],
    public_call_stack: [Field; crate::abi::MAX_PUBLIC_CALL_STACK],
    new_l2_to_l1_msgs: [Field; crate::abi::MAX_L1_MSG_STACK],
    historic_private_data_tree_root: Field,
    historic_private_nullifier_tree_root: Field,
    historic_contract_tree_root: Field,
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
        fields = fields.push_array(self.args);
        fields = fields.push_array(self.return_values);
        fields = fields.push_array(self.emitted_events);
        fields = fields.push_array(self.new_commitments);
        fields = fields.push_array(self.new_nullifiers);
        fields = fields.push_array(self.private_call_stack);
        fields = fields.push_array(self.public_call_stack);
        fields = fields.push_array(self.new_l2_to_l1_msgs);
        fields = fields.push(self.historic_private_data_tree_root);
        fields = fields.push(self.historic_private_nullifier_tree_root);
        fields = fields.push(self.historic_contract_tree_root);
        fields = fields.push_array(self.contract_deployment_data.serialize());
        fields.storage
    }
}
