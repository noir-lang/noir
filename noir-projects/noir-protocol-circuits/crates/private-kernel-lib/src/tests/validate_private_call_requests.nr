use crate::tests::private_call_data_validator_builder::PrivateCallDataValidatorBuilder;

#[test]
fn validate_private_call_requests_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.public_inputs.append_private_call_requests(1, true);
    builder.private_call.public_inputs.append_private_call_requests(1, false);
    builder.private_call.public_inputs.append_private_call_requests(1, true);

    builder.validate();
}

#[test]
fn validate_private_call_requests_from_delegate_call_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_delegate_call();

    builder.private_call.public_inputs.append_private_call_requests(1, true);
    builder.private_call.public_inputs.append_private_call_requests(1, false);
    builder.private_call.public_inputs.append_private_call_requests(1, true);

    builder.validate();
}

#[test]
fn validate_private_call_requests_from_static_call_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_static_call();

    builder.private_call.public_inputs.append_private_call_requests(2, false);

    builder.validate();
}

#[test]
fn validate_private_call_requests_delegate_calls_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.public_inputs.append_private_call_requests(2, true);

    builder.validate();
}

#[test(should_fail_with="invalid caller context")]
fn validate_private_call_requests_incorrect_caller_storage_contract_address_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.public_inputs.append_private_call_requests(1, true);
    let mut call_request = builder.private_call.public_inputs.private_call_requests.pop();
    // Tweak the storage contract to be a different value.
    call_request.caller_context.storage_contract_address.inner += 1;
    builder.private_call.public_inputs.private_call_requests.push(call_request);

    builder.validate();
}

#[test(should_fail_with="invalid caller context")]
fn validate_private_call_requests_incorrect_caller_msg_sender_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.public_inputs.append_private_call_requests(1, true);
    let mut call_request = builder.private_call.public_inputs.private_call_requests.pop();
    // Tweak the msg_sender to be a different value.
    call_request.caller_context.msg_sender.inner += 1;
    builder.private_call.public_inputs.private_call_requests.push(call_request);

    builder.validate();
}

#[test(should_fail_with="mismatch is_static_call flag")]
fn validate_private_call_requests_regular_call_is_static_true_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.public_inputs.append_private_call_requests(1, false);
    let mut call_request = builder.private_call.public_inputs.private_call_requests.pop();
    // Tweak the is_static_call flag to be true.
    call_request.caller_context.is_static_call = true;
    builder.private_call.public_inputs.private_call_requests.push(call_request);

    builder.validate();
}

#[test(should_fail_with="mismatch is_static_call flag")]
fn validate_private_call_requests_static_call_is_static_false_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_static_call();

    builder.private_call.public_inputs.append_private_call_requests(1, false);
    let mut call_request = builder.private_call.public_inputs.private_call_requests.pop();
    // Tweak the is_static_call flag to be false.
    call_request.caller_context.is_static_call = false;
    builder.private_call.public_inputs.private_call_requests.push(call_request);

    builder.validate();
}
