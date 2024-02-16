use dep::protocol_types::{
    abis::{function_selector::FunctionSelector, private_call_stack_item::PrivateCallStackItem},
    address::AztecAddress, constants::PRIVATE_CALL_STACK_ITEM_LENGTH
};

#[oracle(callPrivateFunction)]
fn call_private_function_oracle(
    _contract_address: AztecAddress,
    _function_selector: FunctionSelector,
    _args_hash: Field,
    _start_side_effect_counter: u32,
    _is_static_call: bool,
    _is_delegate_call: bool
) -> [Field; PRIVATE_CALL_STACK_ITEM_LENGTH] {}

unconstrained pub fn call_private_function_internal(
    contract_address: AztecAddress,
    function_selector: FunctionSelector,
    args_hash: Field,
    start_side_effect_counter: u32,
    is_static_call: bool,
    is_delegate_call: bool
) -> PrivateCallStackItem {
    let fields = call_private_function_oracle(
        contract_address,
        function_selector,
        args_hash,
        start_side_effect_counter,
        is_static_call,
        is_delegate_call
    );

    PrivateCallStackItem::deserialize(fields)
}
