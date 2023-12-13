use dep::protocol_types::{
    abis::function_selector::FunctionSelector,
    address::AztecAddress,
};

// contract_address + 
// args_hash +
// crate::abi::FUNCTION_DATA_SIZE +
// crate::abi::CALL_CONTEXT_SIZE +
// = 2 + 4 + 7
global ENQUEUE_PUBLIC_FUNCTION_CALL_RETURN_SIZE: Field = 13;

#[oracle(enqueuePublicFunctionCall)]
fn enqueue_public_function_call_oracle(_contract_address: AztecAddress, _function_selector: FunctionSelector, _args_hash: Field) -> [Field; ENQUEUE_PUBLIC_FUNCTION_CALL_RETURN_SIZE] {}

unconstrained pub fn enqueue_public_function_call_internal(contract_address: AztecAddress, function_selector: FunctionSelector, args_hash: Field) -> [Field; ENQUEUE_PUBLIC_FUNCTION_CALL_RETURN_SIZE] {
    enqueue_public_function_call_oracle(contract_address, function_selector, args_hash)
}
