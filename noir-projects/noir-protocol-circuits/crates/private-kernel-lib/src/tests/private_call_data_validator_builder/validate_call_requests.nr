use crate::{
    components::private_call_data_validator::PrivateCallDataValidator,
    tests::private_call_data_validator_builder::PrivateCallDataValidatorBuilder
};
use dep::types::abis::private_kernel::private_call_data::PrivateCallData;

impl PrivateCallDataValidatorBuilder {
    pub fn get_private_call_data(self) -> PrivateCallData {
        self.private_call.to_private_call_data()
    }

    pub fn validate_with_private_call(private_call: PrivateCallData) {
        PrivateCallDataValidator::new(private_call).validate([]);
    }
}

/**
 * validate_public_call_requests
 */

#[test]
fn validate_public_call_requests_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_public_call_requests(2);

    builder.validate();
}

#[test]
fn validate_public_call_requests_delegate_calls_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_public_call_requests_delegate(2);

    builder.validate();
}

#[test(should_fail_with="call stack hash does not match call request hash")]
fn validate_public_call_requests_incorrect_hash_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_public_call_requests(2);
    let mut private_call = builder.get_private_call_data();
    // Change the hash to be a different value.
    private_call.call_stack_item.public_inputs.public_call_stack_hashes[0] += 1;

    PrivateCallDataValidatorBuilder::validate_with_private_call(private_call);
}

#[test(should_fail_with="invalid caller contract address")]
fn validate_public_call_requests_incorrect_caller_address_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_public_call_requests(1);
    let mut call_request = builder.private_call.public_call_requests.pop();
    // Change the caller contract address to be a different value.
    call_request.caller_contract_address.inner += 1;
    builder.private_call.public_call_requests.push(call_request);

    builder.validate();
}

#[test(should_fail_with="invalid caller context")]
fn validate_public_call_requests_incorrect_caller_storage_contract_address_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_public_call_requests_delegate(1);
    let mut call_request = builder.private_call.public_call_requests.pop();
    // Change the storage contract to be a different value.
    call_request.caller_context.storage_contract_address.inner += 1;
    builder.private_call.public_call_requests.push(call_request);

    builder.validate();
}

#[test(should_fail_with="invalid caller context")]
fn validate_public_call_requests_incorrect_caller_msg_sender_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_public_call_requests_delegate(1);
    let mut call_request = builder.private_call.public_call_requests.pop();
    // Change the msg_sender to be a different value.
    call_request.caller_context.msg_sender.inner += 1;
    builder.private_call.public_call_requests.push(call_request);

    builder.validate();
}

#[test(should_fail_with="call requests length does not match the expected length")]
fn validate_public_call_requests_fewer_hashes_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_public_call_requests(2);
    let mut private_call = builder.get_private_call_data();
    // Remove one call stack item hash.
    private_call.call_stack_item.public_inputs.public_call_stack_hashes[1] = 0;

    PrivateCallDataValidatorBuilder::validate_with_private_call(private_call);
}

#[test(should_fail_with="call stack hash does not match call request hash")]
fn validate_public_call_requests_more_hashes_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_public_call_requests(2);
    let mut private_call = builder.get_private_call_data();
    // Add one random call stack item hash.
    private_call.call_stack_item.public_inputs.public_call_stack_hashes[2] = 123;

    PrivateCallDataValidatorBuilder::validate_with_private_call(private_call);
}

/**
 * validate_teardown_call_request
 */

#[test]
fn validate_teardown_call_request_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_public_teardown_call_request();

    builder.validate();
}

#[test]
fn validate_teardown_call_request_delegate_calls_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_public_teardown_call_request_delegate();

    builder.validate();
}

#[test(should_fail_with="call stack hash does not match call request hash")]
fn validate_teardown_call_request_incorrect_hash_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_public_teardown_call_request_delegate();
    let mut private_call = builder.get_private_call_data();
    // Change the hash to be a different value.
    private_call.call_stack_item.public_inputs.public_teardown_function_hash += 1;

    PrivateCallDataValidatorBuilder::validate_with_private_call(private_call);
}

#[test(should_fail_with="invalid caller contract address")]
fn validate_teardown_call_request_incorrect_caller_address_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_public_teardown_call_request_delegate();
    // Change the caller contract address to be a different value.
    builder.private_call.public_teardown_call_stack.storage[0].caller_contract_address.inner += 1;

    builder.validate();
}

#[test(should_fail_with="invalid caller context")]
fn validate_teardown_call_request_incorrect_caller_storage_contract_address_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_public_teardown_call_request_delegate();
    // Change the storage contract to be a different value.
    builder.private_call.public_teardown_call_stack.storage[0].caller_context.storage_contract_address.inner += 1;

    builder.validate();
}

#[test(should_fail_with="invalid caller context")]
fn validate_teardown_call_request_incorrect_caller_msg_sender_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_public_teardown_call_request_delegate();
    // Change the msg_sender to be a different value.
    builder.private_call.public_teardown_call_stack.storage[0].caller_context.msg_sender.inner += 1;

    builder.validate();
}

#[test(should_fail_with="call requests length does not match the expected length")]
fn validate_teardown_call_request_fewer_hashes_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_public_teardown_call_request_delegate();
    let mut private_call = builder.get_private_call_data();
    // Remove the call stack item hash.
    private_call.call_stack_item.public_inputs.public_teardown_function_hash = 0;

    PrivateCallDataValidatorBuilder::validate_with_private_call(private_call);
}

#[test(should_fail_with="call stack hash does not match call request hash")]
fn validate_teardown_call_request_more_hashes_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_public_teardown_call_request_delegate();
    let mut private_call = builder.get_private_call_data();
    // Add a random call stack item hash.
    private_call.call_stack_item.public_inputs.public_teardown_function_hash = 123;

    PrivateCallDataValidatorBuilder::validate_with_private_call(private_call);
}
