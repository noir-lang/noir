use dep::protocol_types::{abis::function_selector::FunctionSelector, address::AztecAddress, constants::RETURN_VALUES_LENGTH};

#[oracle(callPublicFunction)]
fn call_public_function_oracle(
    _contract_address: AztecAddress,
    _function_selector: FunctionSelector,
    _args_hash: Field,
    _is_static_call: bool,
    _is_delegate_call: bool
) -> [Field; RETURN_VALUES_LENGTH] {}

unconstrained pub fn call_public_function_internal(
    contract_address: AztecAddress,
    function_selector: FunctionSelector,
    args_hash: Field,
    is_static_call: bool,
    is_delegate_call: bool
) -> [Field; RETURN_VALUES_LENGTH] {
    call_public_function_oracle(
        contract_address,
        function_selector,
        args_hash,
        is_static_call,
        is_delegate_call
    )
}
