use dep::protocol_types::{abis::function_selector::FunctionSelector, address::AztecAddress};
#[oracle(callPublicFunction)]
unconstrained fn call_public_function_oracle<let RETURNS_COUNT: u32>(
    _contract_address: AztecAddress,
    _function_selector: FunctionSelector,
    _args_hash: Field,
    _side_effect_counter: u32,
    _is_static_call: bool,
    _is_delegate_call: bool
) -> [Field; RETURNS_COUNT] {}

unconstrained pub fn call_public_function_internal<let RETURNS_COUNT: u32>(
    contract_address: AztecAddress,
    function_selector: FunctionSelector,
    args_hash: Field,
    side_effect_counter: u32,
    is_static_call: bool,
    is_delegate_call: bool
) -> [Field; RETURNS_COUNT] {
    call_public_function_oracle(
        contract_address,
        function_selector,
        args_hash,
        side_effect_counter,
        is_static_call,
        is_delegate_call
    )
}
