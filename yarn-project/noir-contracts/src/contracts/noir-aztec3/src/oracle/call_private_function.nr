
#[oracle(callPrivateFunction)]
fn call_private_function_oracle(
    _contract_address: Field, 
    _function_selector: Field, 
    _args: [Field; crate::abi::MAX_ARGS]
) -> [Field; 42] {}

unconstrained fn call_private_function_internal(
    contract_address: Field, 
    function_selector: Field,
    args: [Field; crate::abi::MAX_ARGS]
) -> [Field; 42] {
    call_private_function_oracle(
        contract_address, 
        function_selector, 
        args,
    )
}