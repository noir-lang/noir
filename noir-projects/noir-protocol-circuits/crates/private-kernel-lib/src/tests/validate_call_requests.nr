use crate::tests::private_call_data_validator_builder::PrivateCallDataValidatorBuilder;
use dep::types::abis::call_request::CallRequest;

/**
 * validate_public_call_requests
 */

#[test]
fn validate_public_call_requests_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_public_call_requests(2, false);

    builder.validate();
}

#[test]
fn validate_public_call_requests_delegate_calls_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_public_call_requests(2, true);

    builder.validate();
}

#[test(should_fail_with="call stack hash does not match call request hash")]
fn validate_public_call_requests_incorrect_hash_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_public_call_requests(2, false);
    let mut call_request = builder.private_call.public_call_stack.pop();
    // Change the hash to be a different value.
    call_request.hash += 1;
    builder.private_call.public_call_stack.push(call_request);

    builder.validate();
}

#[test(should_fail_with="invalid caller contract address")]
fn validate_public_call_requests_incorrect_caller_address_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_public_call_requests(1, false);
    let mut call_request = builder.private_call.public_call_stack.pop();
    // Change the caller contract address to be a different value.
    call_request.caller_contract_address.inner += 1;
    builder.private_call.public_call_stack.push(call_request);

    builder.validate();
}

#[test(should_fail_with="invalid caller context")]
fn validate_public_call_requests_incorrect_caller_storage_contract_address_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_public_call_requests(1, true);
    let mut call_request = builder.private_call.public_call_stack.pop();
    // Change the storage contract to be a different value.
    call_request.caller_context.storage_contract_address.inner += 1;
    builder.private_call.public_call_stack.push(call_request);

    builder.validate();
}

#[test(should_fail_with="invalid caller context")]
fn validate_public_call_requests_incorrect_caller_msg_sender_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_public_call_requests(1, true);
    let mut call_request = builder.private_call.public_call_stack.pop();
    // Change the msg_sender to be a different value.
    call_request.caller_context.msg_sender.inner += 1;
    builder.private_call.public_call_stack.push(call_request);

    builder.validate();
}

#[test(should_fail_with="call requests length does not match the expected length")]
fn validate_public_call_requests_fewer_hashes_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_public_call_requests(2, false);
    // Remove one call stack item hash.
    let _ = builder.private_call.public_inputs.public_call_stack_hashes.pop();

    builder.validate();
}

#[test(should_fail_with="call stack hash does not match call request hash")]
fn validate_public_call_requests_more_hashes_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_public_call_requests(2, false);
    // Add one random call stack item hash.
    builder.private_call.public_inputs.public_call_stack_hashes.push(9123);

    builder.validate();
}

/**
 * validate_teardown_call_request
 */

#[test]
fn validate_teardown_call_request_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.add_teardown_call_request(false);

    builder.validate();
}

#[test]
fn validate_teardown_call_request_delegate_calls_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.add_teardown_call_request(true);

    builder.validate();
}

#[test(should_fail_with="call stack hash does not match call request hash")]
fn validate_teardown_call_request_incorrect_hash_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.add_teardown_call_request(true);
    // Change the hash to be a different value.
    builder.private_call.public_teardown_call_request.hash += 1;

    builder.validate();
}

#[test(should_fail_with="invalid caller contract address")]
fn validate_teardown_call_request_incorrect_caller_address_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.add_teardown_call_request(true);
    // Change the caller contract address to be a different value.
    builder.private_call.public_teardown_call_request.caller_contract_address.inner += 1;

    builder.validate();
}

#[test(should_fail_with="invalid caller context")]
fn validate_teardown_call_request_incorrect_caller_storage_contract_address_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.add_teardown_call_request(true);
    // Change the storage contract to be a different value.
    builder.private_call.public_teardown_call_request.caller_context.storage_contract_address.inner += 1;

    builder.validate();
}

#[test(should_fail_with="invalid caller context")]
fn validate_teardown_call_request_incorrect_caller_msg_sender_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.add_teardown_call_request(true);
    // Change the msg_sender to be a different value.
    builder.private_call.public_teardown_call_request.caller_context.msg_sender.inner += 1;

    builder.validate();
}

#[test(should_fail_with="call requests length does not match the expected length")]
fn validate_teardown_call_request_fewer_hashes_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.add_teardown_call_request(true);
    // Remove the call stack item hash.
    builder.private_call.public_inputs.public_teardown_function_hash = 0;

    builder.validate();
}

#[test(should_fail_with="call stack hash does not match call request hash")]
fn validate_teardown_call_request_more_hashes_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.add_teardown_call_request(true);
    // Remove the call request.
    builder.private_call.public_teardown_call_request = CallRequest::empty();

    builder.validate();
}
