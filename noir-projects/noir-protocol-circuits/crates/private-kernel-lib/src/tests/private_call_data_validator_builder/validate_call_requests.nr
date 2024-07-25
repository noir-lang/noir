use crate::tests::private_call_data_validator_builder::PrivateCallDataValidatorBuilder;

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
fn validate_public_call_requests_delegate_call_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_public_call_requests_delegate(2);

    builder.validate();
}

#[test]
fn validate_public_call_requests_mix_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_public_call_requests(1);
    builder.private_call.append_public_call_requests_delegate(1);
    builder.private_call.append_public_call_requests(1);

    builder.validate();
}

#[test]
fn validate_public_call_requests_from_static_call_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_static_call();

    builder.private_call.append_public_call_requests(1);
    builder.private_call.append_public_call_requests_delegate(1);
    builder.private_call.append_public_call_requests(1);

    builder.validate();
}

#[test(should_fail_with="incorrect msg_sender for delegate call request")]
fn validate_public_call_requests_incorrect_msg_sender_for_delegate_call_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_public_call_requests_delegate(1);
    // Change the msg_sender to be the contract address.
    builder.private_call.public_call_requests.storage[0].item.call_context.msg_sender = builder.private_call.contract_address;

    builder.validate();
}

#[test(should_fail_with="incorrect storage_contract_address for delegate call request")]
fn validate_public_call_requests_incorrect_storage_contract_address_for_delegate_call_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_public_call_requests_delegate(1);
    // Change the storage_contract_address to be the target contract address.
    let target_contract = builder.private_call.public_call_requests.storage[0].item.contract_address;
    builder.private_call.public_call_requests.storage[0].item.call_context.storage_contract_address = target_contract;

    builder.validate();
}

#[test(should_fail_with="incorrect msg_sender for call request")]
fn validate_public_call_requests_incorrect_msg_sender_for_regular_call_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_public_call_requests(1);
    // Change the msg_sender to be the caller's msg_sender.
    builder.private_call.public_call_requests.storage[0].item.call_context.msg_sender = builder.private_call.msg_sender;

    builder.validate();
}

#[test(should_fail_with="incorrect storage_contract_address for call request")]
fn validate_public_call_requests_incorrect_storage_contract_address_for_regular_call_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_public_call_requests(1);
    // Change the storage_contract_address to be the caller's storage contract address.
    builder.private_call.public_call_requests.storage[0].item.call_context.storage_contract_address = builder.private_call.storage_contract_address;

    builder.validate();
}

#[test(should_fail_with="static call cannot make non-static calls")]
fn validate_public_call_requests_static_call_regular_call_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_static_call();

    builder.private_call.append_public_call_requests(1);
    // Tweak the request to be making a non-static call.
    builder.private_call.public_call_requests.storage[0].item.call_context.is_static_call = false;

    builder.validate();
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
fn validate_teardown_call_request_delegate_call_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_public_teardown_call_request_delegate();

    builder.validate();
}

#[test]
fn validate_teardown_call_request_from_static_call_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_static_call();

    builder.private_call.append_public_teardown_call_request();

    builder.validate();
}

#[test]
fn validate_teardown_call_request_delegate_from_static_call_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_static_call();

    builder.private_call.append_public_teardown_call_request_delegate();

    builder.validate();
}

#[test(should_fail_with="incorrect msg_sender for delegate call request")]
fn validate_teardown_call_request_incorrect_msg_sender_for_delegate_call_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_public_teardown_call_request_delegate();
    // Change the msg_sender to be the contract address.
    builder.private_call.public_teardown_call_stack.storage[0].item.call_context.msg_sender = builder.private_call.contract_address;

    builder.validate();
}

#[test(should_fail_with="incorrect storage_contract_address for delegate call request")]
fn validate_teardown_call_request_incorrect_storage_contract_address_for_delegate_call_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_public_teardown_call_request_delegate();
    // Change the storage_contract_address to be the target contract address.
    let target_contract = builder.private_call.public_teardown_call_stack.storage[0].item.contract_address;
    builder.private_call.public_teardown_call_stack.storage[0].item.call_context.storage_contract_address = target_contract;

    builder.validate();
}

#[test(should_fail_with="incorrect msg_sender for call request")]
fn validate_teardown_call_request_incorrect_msg_sender_for_regular_call_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_public_teardown_call_request();
    // Change the msg_sender to be the caller's msg_sender.
    builder.private_call.public_teardown_call_stack.storage[0].item.call_context.msg_sender = builder.private_call.msg_sender;

    builder.validate();
}

#[test(should_fail_with="incorrect storage_contract_address for call request")]
fn validate_teardown_call_request_incorrect_storage_contract_address_for_regular_call_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_public_teardown_call_request();
    // Change the storage_contract_address to be the caller's storage contract address.
    builder.private_call.public_teardown_call_stack.storage[0].item.call_context.storage_contract_address = builder.private_call.storage_contract_address;

    builder.validate();
}

#[test(should_fail_with="static call cannot make non-static calls")]
fn validate_teardown_call_request_static_call_regular_call_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_static_call();

    builder.private_call.append_public_teardown_call_request();
    // Tweak the request to be making a non-static call.
    builder.private_call.public_teardown_call_stack.storage[0].item.call_context.is_static_call = false;

    builder.validate();
}
