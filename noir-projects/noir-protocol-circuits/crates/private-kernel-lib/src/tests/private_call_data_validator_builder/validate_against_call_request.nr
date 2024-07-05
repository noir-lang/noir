use crate::tests::private_call_data_validator_builder::PrivateCallDataValidatorBuilder;
use dep::types::abis::caller_context::CallerContext;

#[test]
fn validate_against_call_request_succeeds() {
    let builder = PrivateCallDataValidatorBuilder::new();

    let request = builder.private_call.build_private_call_request();

    builder.validate_against_call_request(request);
}

#[test]
fn validate_against_call_request_delegate_call_succeeds() {
    let builder = PrivateCallDataValidatorBuilder::new().is_delegate_call();

    let request = builder.private_call.build_private_call_request();

    builder.validate_against_call_request(request);
}

#[test]
fn validate_against_call_request_static_call_succeeds() {
    let builder = PrivateCallDataValidatorBuilder::new().is_static_call();

    let request = builder.private_call.build_private_call_request();

    builder.validate_against_call_request(request);
}

#[test(should_fail_with="calculated private_call_hash does not match provided private_call_hash at the top of the call stack")]
fn validate_against_call_request_mismatch_hash_fails() {
    let builder = PrivateCallDataValidatorBuilder::new();

    let mut request = builder.private_call.build_private_call_request();
    // Tweak the hash to be a different value.
    request.call_request.args_hash += 1;

    builder.validate_against_call_request(request);
}

#[test(should_fail_with="caller context cannot be hidden for delegate calls")]
fn validate_against_call_request_empty_caller_context_for_delegate_calls_fails() {
    let builder = PrivateCallDataValidatorBuilder::new().is_delegate_call();

    let mut request = builder.private_call.build_private_call_request();
    request.call_request.caller_context = CallerContext::empty();

    builder.validate_against_call_request(request);
}

#[test(should_fail_with="call stack msg_sender does not match expected msg_sender for delegate calls")]
fn validate_against_call_request_incorrect_msg_sender_for_delegate_call_fails() {
    let builder = PrivateCallDataValidatorBuilder::new().is_delegate_call();

    let mut request = builder.private_call.build_private_call_request();
    // Tweak the msg_sender to be a different value.
    request.call_request.caller_context.msg_sender.inner += 1;

    builder.validate_against_call_request(request);
}

#[test(should_fail_with="call stack storage address does not match expected contract address for delegate calls")]
fn validate_against_call_request_incorrect_storage_contract_address_for_delegate_call_fails() {
    let builder = PrivateCallDataValidatorBuilder::new().is_delegate_call();

    let mut request = builder.private_call.build_private_call_request();
    // Tweak the storage contract address to be a different value.
    request.call_request.caller_context.storage_contract_address.inner += 1;

    builder.validate_against_call_request(request);
}

#[test(should_fail_with="call stack msg_sender does not match caller contract address")]
fn validate_against_call_request_incorrect_msg_sender_for_regular_call_fails() {
    let builder = PrivateCallDataValidatorBuilder::new();

    let mut request = builder.private_call.build_private_call_request();
    // Tweak the caller's contract address to be a different value.
    request.contract_address.inner += 1;

    builder.validate_against_call_request(request);
}

#[test(should_fail_with="caller context must be hidden for non-delegate calls")]
fn validate_against_call_request_non_empty_caller_for_regular_call_fails() {
    let builder = PrivateCallDataValidatorBuilder::new();

    let mut request = builder.private_call.build_private_call_request();
    // Tweak the caller's msg_sender to be non-empty.
    request.call_request.caller_context.msg_sender.inner = 1234;

    builder.validate_against_call_request(request);
}

#[test(should_fail_with="caller context must be hidden for non-delegate calls")]
fn validate_against_call_request_non_empty_caller_for_static_call_fails() {
    let builder = PrivateCallDataValidatorBuilder::new().is_static_call();

    let mut request = builder.private_call.build_private_call_request();
    // Tweak the caller's msg_sender to be non-empty.
    request.call_request.caller_context.msg_sender.inner = 1234;

    builder.validate_against_call_request(request);
}

#[test(should_fail_with="static call cannot make non-static calls")]
fn validate_against_call_request_static_call_regular_call_fails() {
    let builder = PrivateCallDataValidatorBuilder::new();

    let mut request = builder.private_call.build_private_call_request();
    // Tweak the caller to be making a static call.
    request.call_request.caller_context.is_static_call = true;

    builder.validate_against_call_request(request);
}

#[test(should_fail_with="static call cannot make non-static calls")]
fn validate_against_call_request_static_call_delegate_call_fails() {
    let builder = PrivateCallDataValidatorBuilder::new().is_delegate_call();

    let mut request = builder.private_call.build_private_call_request();
    // Tweak the caller to be making a static call.
    request.call_request.caller_context.is_static_call = true;

    builder.validate_against_call_request(request);
}
