use crate::tests::private_call_data_validator_builder::PrivateCallDataValidatorBuilder;

#[test]
fn validate_against_call_request_succeeds() {
    let builder = PrivateCallDataValidatorBuilder::new();

    let request = builder.private_call.build_private_call_request();

    builder.validate_against_call_request(request);
}

#[test]
fn validate_against_call_request_static_call_succeeds() {
    let builder = PrivateCallDataValidatorBuilder::new().is_static_call();

    let request = builder.private_call.build_private_call_request();

    builder.validate_against_call_request(request);
}

#[test(should_fail_with = "call_context does not match call request")]
fn validate_against_call_request_mismatch_call_context_contract_address_fails() {
    let builder = PrivateCallDataValidatorBuilder::new();

    let mut request = builder.private_call.build_private_call_request();
    // Tweak the contract_address to be a different value.
    request.call_context.contract_address.inner += 1;

    builder.validate_against_call_request(request);
}

#[test(should_fail_with = "args_hash does not match call request")]
fn validate_against_call_request_mismatch_args_hash_fails() {
    let builder = PrivateCallDataValidatorBuilder::new();

    let mut request = builder.private_call.build_private_call_request();
    // Tweak the args_hash to be a different value.
    request.args_hash += 1;

    builder.validate_against_call_request(request);
}

#[test(should_fail_with = "returns_hash does not match call request")]
fn validate_against_call_request_mismatch_returns_hash_fails() {
    let builder = PrivateCallDataValidatorBuilder::new();

    let mut request = builder.private_call.build_private_call_request();
    // Tweak the returns_hash to be a different value.
    request.returns_hash += 1;

    builder.validate_against_call_request(request);
}

#[test(should_fail_with = "start_side_effect_counter does not match call request")]
fn validate_against_call_request_mismatch_start_side_effect_counter_fails() {
    let builder = PrivateCallDataValidatorBuilder::new();

    let mut request = builder.private_call.build_private_call_request();
    // Tweak the start_side_effect_counter to be a different value.
    request.start_side_effect_counter += 1;

    builder.validate_against_call_request(request);
}

#[test(should_fail_with = "end_side_effect_counter does not match call request")]
fn validate_against_call_request_mismatch_end_side_effect_counter_fails() {
    let builder = PrivateCallDataValidatorBuilder::new();

    let mut request = builder.private_call.build_private_call_request();
    // Tweak the end_side_effect_counter to be a different value.
    request.end_side_effect_counter += 1;

    builder.validate_against_call_request(request);
}
