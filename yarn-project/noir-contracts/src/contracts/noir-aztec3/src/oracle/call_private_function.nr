// Size of the return value of a private function call, public inputs length + 5
// contract address, function selector, is_private, is_constructor, public inputs (43), is_execution_request
global CALL_PRIVATE_FUNCTION_RETURN_SIZE: comptime Field = 48;

#[oracle(callPrivateFunction)]
fn call_private_function_oracle(
    _contract_address: Field,
    _function_selector: Field,
    _args: [Field; crate::abi::MAX_ARGS]
) -> [Field; CALL_PRIVATE_FUNCTION_RETURN_SIZE] {}

unconstrained fn call_private_function_internal(
    contract_address: Field,
    function_selector: Field,
    args: [Field; crate::abi::MAX_ARGS]
) -> [Field; CALL_PRIVATE_FUNCTION_RETURN_SIZE] {
    call_private_function_oracle(
        contract_address,
        function_selector,
        args,
    )
}