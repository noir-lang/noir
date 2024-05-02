use dep::protocol_types::{abis::function_selector::FunctionSelector, address::AztecAddress};
// TODO(6052): get new side effect counter from this call
#[oracle(callPublicFunction)]
fn call_public_function_oracle<RETURNS_COUNT>(
    _contract_address: AztecAddress,
    _function_selector: FunctionSelector,
    _args_hash: Field,
    _side_effect_counter: u32,
    _is_static_call: bool,
    _is_delegate_call: bool
) -> [Field; RETURNS_COUNT] {}

unconstrained pub fn call_public_function_internal<RETURNS_COUNT>(
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
