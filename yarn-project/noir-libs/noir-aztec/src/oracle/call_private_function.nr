// Size of the return value of a private function call, public inputs length + 5
// contract address, function selector, is_private, is_constructor, public inputs (53), is_execution_request
global CALL_PRIVATE_FUNCTION_RETURN_SIZE: comptime Field = 58;

#[oracle(callPrivateFunction)]
fn call_private_function_oracle(
    _contract_address: Field,
    _function_selector: Field,
    _args_hash: Field
) -> [Field; CALL_PRIVATE_FUNCTION_RETURN_SIZE] {}

unconstrained fn call_private_function_internal(
    contract_address: Field,
    function_selector: Field,
    args_hash: Field
) -> [Field; CALL_PRIVATE_FUNCTION_RETURN_SIZE] {
    call_private_function_oracle(
        contract_address,
        function_selector,
        args_hash,
    )
}