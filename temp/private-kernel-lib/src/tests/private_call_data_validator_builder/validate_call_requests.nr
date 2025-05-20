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
fn validate_public_call_requests_from_static_call_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_static_call();

    builder.private_call.append_public_call_requests(2);

    builder.validate();
}

#[test(should_fail_with = "incorrect msg_sender for call request")]
fn validate_public_call_requests_incorrect_msg_sender_for_regular_call_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_public_call_requests(1);
    // Change the msg_sender to be the caller's msg_sender.
    let mut call_request = builder.private_call.public_call_requests.get(0);
    call_request.inner.msg_sender = builder.private_call.msg_sender;
    builder.private_call.public_call_requests.set(0, call_request);

    builder.validate();
}

#[test(should_fail_with = "static call cannot make non-static calls")]
fn validate_public_call_requests_static_call_regular_call_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_static_call();

    builder.private_call.append_public_call_requests(1);
    // Tweak the request to be making a non-static call.
    let mut call_request = builder.private_call.public_call_requests.get(0);
    call_request.inner.is_static_call = false;
    builder.private_call.public_call_requests.set(0, call_request);

    builder.validate();
}

/**
 * validate_teardown_call_request
 */
#[test]
fn validate_teardown_call_request_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.set_public_teardown_call_request();

    builder.validate();
}

#[test]
fn validate_teardown_call_request_from_static_call_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_static_call();

    builder.private_call.set_public_teardown_call_request();

    builder.validate();
}

#[test(should_fail_with = "incorrect msg_sender for call request")]
fn validate_teardown_call_request_incorrect_msg_sender_for_regular_call_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.set_public_teardown_call_request();
    // Change the msg_sender to be the caller's msg_sender.
    builder.private_call.public_teardown_call_request.msg_sender = builder.private_call.msg_sender;

    builder.validate();
}

#[test(should_fail_with = "static call cannot make non-static calls")]
fn validate_teardown_call_request_static_call_regular_call_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_static_call();

    builder.private_call.set_public_teardown_call_request();
    // Tweak the request to be making a non-static call.
    builder.private_call.public_teardown_call_request.is_static_call = false;

    builder.validate();
}
