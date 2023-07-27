
// contract_address + 
// args_hash +
// crate::abi::FUNCTION_DATA_SIZE +
// crate::abi::CALL_CONTEXT_SIZE +
// = 2 + 4 + 6
global ENQUEUE_PUBLIC_FUNCTION_CALL_RETURN_SIZE: comptime Field = 12;

#[oracle(enqueuePublicFunctionCall)]
fn enqueue_public_function_call_oracle(
    _contract_address: Field, 
    _function_selector: Field, 
    _args_hash: Field,
) -> [Field; ENQUEUE_PUBLIC_FUNCTION_CALL_RETURN_SIZE] {}

unconstrained fn enqueue_public_function_call_internal(
    contract_address: Field, 
    function_selector: Field,
    args_hash: Field
) -> [Field; ENQUEUE_PUBLIC_FUNCTION_CALL_RETURN_SIZE] {
    enqueue_public_function_call_oracle(
        contract_address, 
        function_selector, 
        args_hash,
    )
}