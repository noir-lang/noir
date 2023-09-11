use crate::constants_gen::RETURN_VALUES_LENGTH;

#[oracle(callPublicFunction)]
fn call_public_function_oracle(
    _contract_address: Field, 
    _function_selector: Field, 
    _args_hash: Field
) -> [Field; RETURN_VALUES_LENGTH] {}

unconstrained fn call_public_function_internal(
    contract_address: Field, 
    function_selector: Field,
    args_hash: Field
) -> [Field; RETURN_VALUES_LENGTH] {
    call_public_function_oracle(
        contract_address, 
        function_selector, 
        args_hash,
    )
}
