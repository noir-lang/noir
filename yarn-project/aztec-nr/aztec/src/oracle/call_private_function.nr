use dep::protocol_types::{
    abis::function_selector::FunctionSelector,
    address::AztecAddress,
    constants::CALL_PRIVATE_FUNCTION_RETURN_SIZE,
};

#[oracle(callPrivateFunction)]
fn call_private_function_oracle(_contract_address: AztecAddress, _function_selector: FunctionSelector, _args_hash: Field) -> [Field; CALL_PRIVATE_FUNCTION_RETURN_SIZE] {}

unconstrained pub fn call_private_function_internal(contract_address: AztecAddress, function_selector: FunctionSelector, args_hash: Field) -> [Field; CALL_PRIVATE_FUNCTION_RETURN_SIZE] {
    call_private_function_oracle(contract_address, function_selector, args_hash)
}
