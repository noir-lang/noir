#[oracle(callPublicFunction)]
fn call_public_function_oracle(
    _contract_address: Field, 
    _function_selector: Field, 
    _args_hash: Field
) -> [Field; crate::abi::MAX_RETURN_VALUES] {}

unconstrained fn call_public_function_internal(
    contract_address: Field, 
    function_selector: Field,
    args_hash: Field
) -> [Field; crate::abi::MAX_RETURN_VALUES] {
    call_public_function_oracle(
        contract_address, 
        function_selector, 
        args_hash,
    )
}
