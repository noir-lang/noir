use crate::tests::private_call_data_validator_builder::PrivateCallDataValidatorBuilder;

#[test]
fn validate_against_tx_request_succeeds() {
    let builder = PrivateCallDataValidatorBuilder::new();

    let request = builder.private_call.build_tx_request();

    builder.validate_against_tx_request(request);
}

#[test(should_fail_with="origin address does not match call stack items contract address")]
fn validate_against_tx_request_mismatch_contract_address_fails() {
    let builder = PrivateCallDataValidatorBuilder::new();

    let mut request = builder.private_call.build_tx_request();
    // Tweak the origin to be a different value.
    request.origin.inner += 1;

    builder.validate_against_tx_request(request);
}

#[test(should_fail_with="tx_request function_data must match call_stack_item function_data")]
fn validate_against_tx_request_mismatch_function_data_fails() {
    let builder = PrivateCallDataValidatorBuilder::new();

    let mut request = builder.private_call.build_tx_request();
    // Tweak the function selector to be a different value.
    request.function_data.selector.inner += 1;

    builder.validate_against_tx_request(request);
}

#[test(should_fail_with="noir function args passed to tx_request must match args in the call_stack_item")]
fn validate_against_tx_request_mismatch_args_hash_fails() {
    let builder = PrivateCallDataValidatorBuilder::new();

    let mut request = builder.private_call.build_tx_request();
    // Tweak the args hash to be a different value.
    request.args_hash += 1;

    builder.validate_against_tx_request(request);
}

#[test(should_fail_with="tx_context in tx_request must match tx_context in call_stack_item")]
fn validate_against_tx_request_mismatch_chain_id_fails() {
    let builder = PrivateCallDataValidatorBuilder::new();

    let mut request = builder.private_call.build_tx_request();
    // Tweak the chain id to be a different value.
    request.tx_context.chain_id += 1;

    builder.validate_against_tx_request(request);
}

#[test(should_fail_with="tx_context in tx_request must match tx_context in call_stack_item")]
fn validate_against_tx_request_mismatch_version_fails() {
    let builder = PrivateCallDataValidatorBuilder::new();

    let mut request = builder.private_call.build_tx_request();
    // Tweak the version to be a different value.
    request.tx_context.version += 1;

    builder.validate_against_tx_request(request);
}
