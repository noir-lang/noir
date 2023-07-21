use crate::constants_gen::CALL_PRIVATE_FUNCTION_RETURN_SIZE;

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