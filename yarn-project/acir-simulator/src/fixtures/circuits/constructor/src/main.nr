use dep::abi;

fn constrain_call_contexts(a: abi::CallContext, b: abi::CallContext) {
    constrain a.msg_sender == b.msg_sender;
    constrain a.storage_contract_address == b.storage_contract_address;
    constrain a.portal_contract_address == b.portal_contract_address;

    constrain a.is_delegate_call == b.is_delegate_call;
    constrain a.is_static_call == b.is_static_call;
    constrain a.is_contract_deployment == b.is_contract_deployment;
}

fn constrain_contract_deployment_data(a: abi::ContractDeploymentData, b: abi::ContractDeploymentData) {
    constrain a.constructor_vk_hash == b.constructor_vk_hash;
    constrain a.function_tree_root == b.function_tree_root;
    constrain a.contract_address_salt == b.contract_address_salt;
    constrain a.portal_contract_address == b.portal_contract_address;
    constrain a.hide_private_function_data == b.hide_private_function_data;
}

fn main(
    public_inputs: pub abi::PublicInputs,
    args: [Field; abi::MAX_ARGS], 
    call_context: abi::CallContext, 
    contract_deployment_data: abi::ContractDeploymentData
    ) {
    constrain_call_contexts(public_inputs.call_context, call_context);

    constrain public_inputs.args == args;
    constrain public_inputs.return_values == [0; abi::MAX_RETURN_VALUES];
    constrain public_inputs.logs == [0; abi::MAX_LOGS];

    constrain public_inputs.new_commitments == [0; abi::MAX_NEW_COMMITMENTS];
    constrain public_inputs.new_nullifiers == [0; abi::MAX_NEW_NULLIFIERS];

    constrain public_inputs.private_call_stack == [0; abi::MAX_PRIVATE_CALL_STACK];
    constrain public_inputs.public_call_stack == [0; abi::MAX_PUBLIC_CALL_STACK];
    constrain public_inputs.l1_message_stack == [0; abi::MAX_L1_MSG_STACK];

    constrain public_inputs.old_private_data_tree_root == 0;
    constrain public_inputs.old_nullifier_tree_root == 0;
    constrain public_inputs.old_contract_tree_root == 0;

    constrain_contract_deployment_data(public_inputs.contract_deployment_data, contract_deployment_data);
}
