use dep::protocol_types::{
    abis::{function_selector::FunctionSelector, private_call_stack_item::PrivateCallStackItem},
    address::AztecAddress, constants::PRIVATE_CALL_STACK_ITEM_LENGTH
};

#[oracle(callPrivateFunction)]
fn call_private_function_oracle(
    _contract_address: AztecAddress,
    _function_selector: FunctionSelector,
    _args_hash: Field,
    _start_side_effect_counter: u32
) -> [Field; PRIVATE_CALL_STACK_ITEM_LENGTH] {}

unconstrained pub fn call_private_function_internal(
    contract_address: AztecAddress,
    function_selector: FunctionSelector,
    args_hash: Field,
    start_side_effect_counter: u32
) -> PrivateCallStackItem {
    let fields = call_private_function_oracle(
        contract_address,
        function_selector,
        args_hash,
        start_side_effect_counter
    );

    PrivateCallStackItem::deserialize(fields)
}
