use dep::protocol_types::{abis::function_selector::FunctionSelector, address::AztecAddress, utils::reader::Reader};

#[oracle(callPrivateFunction)]
unconstrained fn call_private_function_oracle(
    _contract_address: AztecAddress,
    _function_selector: FunctionSelector,
    _args_hash: Field,
    _start_side_effect_counter: u32,
    _is_static_call: bool,
    _is_delegate_call: bool
) -> [Field; 2] {}

unconstrained pub fn call_private_function_internal(
    contract_address: AztecAddress,
    function_selector: FunctionSelector,
    args_hash: Field,
    start_side_effect_counter: u32,
    is_static_call: bool,
    is_delegate_call: bool
) -> (u32, Field) {
    let fields = call_private_function_oracle(
        contract_address,
        function_selector,
        args_hash,
        start_side_effect_counter,
        is_static_call,
        is_delegate_call
    );

    let mut reader = Reader::new(fields);
    let end_side_effect_counter = reader.read_u32();
    let returns_hash = reader.read();

    (end_side_effect_counter, returns_hash)
}
