use dep::protocol_types::{abis::{function_selector::FunctionSelector}, address::AztecAddress};

#[oracle(enqueuePublicFunctionCall)]
unconstrained fn enqueue_public_function_call_oracle(
    _contract_address: AztecAddress,
    _function_selector: FunctionSelector,
    _args_hash: Field,
    _side_effect_counter: u32,
    _is_static_call: bool,
    _is_delegate_call: bool
) {}

unconstrained pub fn enqueue_public_function_call_internal(
    contract_address: AztecAddress,
    function_selector: FunctionSelector,
    args_hash: Field,
    side_effect_counter: u32,
    is_static_call: bool,
    is_delegate_call: bool
) {
    enqueue_public_function_call_oracle(
        contract_address,
        function_selector,
        args_hash,
        side_effect_counter,
        is_static_call,
        is_delegate_call
    );
}

#[oracle(setPublicTeardownFunctionCall)]
unconstrained fn set_public_teardown_function_call_oracle(
    _contract_address: AztecAddress,
    _function_selector: FunctionSelector,
    _args_hash: Field,
    _side_effect_counter: u32,
    _is_static_call: bool,
    _is_delegate_call: bool
) {}

unconstrained pub fn set_public_teardown_function_call_internal(
    contract_address: AztecAddress,
    function_selector: FunctionSelector,
    args_hash: Field,
    side_effect_counter: u32,
    is_static_call: bool,
    is_delegate_call: bool
) {
    set_public_teardown_function_call_oracle(
        contract_address,
        function_selector,
        args_hash,
        side_effect_counter,
        is_static_call,
        is_delegate_call
    );
}

