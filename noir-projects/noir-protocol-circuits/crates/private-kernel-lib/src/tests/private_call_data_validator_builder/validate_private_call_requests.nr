use crate::tests::private_call_data_validator_builder::PrivateCallDataValidatorBuilder;
use dep::types::constants::MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL;

impl PrivateCallDataValidatorBuilder {
    pub fn new_first_call() -> Self {
        PrivateCallDataValidatorBuilder::new_from_counter(0)
    }

    pub fn split_calls(&mut self, counter: u32) {
        self.private_call.min_revertible_side_effect_counter = counter;
    }

    pub fn add_private_call_request(&mut self, counter_start: u32, counter_end: u32) {
        let index = self.private_call.private_call_requests.len();
        self.private_call.append_private_call_requests(1);
        self.private_call.private_call_requests.storage[index].call_request.start_side_effect_counter = counter_start;
        self.private_call.private_call_requests.storage[index].call_request.end_side_effect_counter = counter_end;
        self.private_call.counter = counter_end + 1;
    }
}

#[test]
fn validate_private_call_requests_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_private_call_requests(1);
    builder.private_call.append_private_call_requests_delegate(1);
    builder.private_call.append_private_call_requests(1);

    builder.validate();
}

#[test]
fn validate_private_call_requests_from_delegate_call_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_delegate_call();

    builder.private_call.append_private_call_requests(1);
    builder.private_call.append_private_call_requests_delegate(1);
    builder.private_call.append_private_call_requests(1);

    builder.validate();
}

#[test]
fn validate_private_call_requests_from_static_call_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_static_call();

    builder.private_call.append_private_call_requests(1);
    builder.private_call.append_private_call_requests_delegate(1);
    builder.private_call.append_private_call_requests(1);

    builder.validate();
}

#[test(should_fail_with="invalid caller context")]
fn validate_private_call_requests_incorrect_caller_storage_contract_address_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_private_call_requests_delegate(1);
    let mut call_request = builder.private_call.private_call_requests.pop();
    // Tweak the storage contract to be a different value.
    call_request.call_request.caller_context.storage_contract_address.inner += 1;
    builder.private_call.private_call_requests.push(call_request);

    builder.validate();
}

#[test(should_fail_with="invalid caller context")]
fn validate_private_call_requests_incorrect_caller_msg_sender_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_private_call_requests_delegate(1);
    let mut call_request = builder.private_call.private_call_requests.pop();
    // Tweak the msg_sender to be a different value.
    call_request.call_request.caller_context.msg_sender.inner += 1;
    builder.private_call.private_call_requests.push(call_request);

    builder.validate();
}

#[test(should_fail_with="mismatch is_static_call flag")]
fn validate_private_call_requests_regular_call_is_static_true_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_private_call_requests(1);
    let mut call_request = builder.private_call.private_call_requests.pop();
    // Tweak the is_static_call flag to be true.
    call_request.call_request.caller_context.is_static_call = true;
    builder.private_call.private_call_requests.push(call_request);

    builder.validate();
}

#[test(should_fail_with="mismatch is_static_call flag")]
fn validate_private_call_requests_static_call_is_static_false_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_static_call();

    builder.private_call.append_private_call_requests(1);
    let mut call_request = builder.private_call.private_call_requests.pop();
    // Tweak the is_static_call flag to be false.
    call_request.call_request.caller_context.is_static_call = false;
    builder.private_call.private_call_requests.push(call_request);

    builder.validate();
}

/**
 * Splitting call requests.
 */

#[test]
fn validate_private_call_requests_split_private_calls_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new_first_call();

    builder.add_private_call_request(20, 30);
    builder.add_private_call_request(40, 50);
    builder.split_calls(60);
    builder.add_private_call_request(60, 70);

    builder.validate();
}

#[test]
fn validate_private_call_requests_split_private_calls_empty_revertible_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new_first_call();

    builder.add_private_call_request(20, 30);
    builder.add_private_call_request(40, 50);
    builder.split_calls(51);

    builder.validate();
}

#[test]
fn validate_private_call_requests_split_private_calls_empty_non_revertible_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new_first_call();

    builder.split_calls(20);
    builder.add_private_call_request(20, 30);
    builder.add_private_call_request(40, 50);

    builder.validate();
}

#[test]
fn validate_private_call_requests_split_private_calls_full_non_revertible_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new_first_call();

    builder.private_call.append_private_call_requests(MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL);
    builder.split_calls(builder.private_call.counter);

    builder.validate();
}

#[test]
fn validate_private_call_requests_split_private_calls_less_than_first_revertible_success() {
    let mut builder = PrivateCallDataValidatorBuilder::new_first_call();

    builder.add_private_call_request(20, 30);
    builder.add_private_call_request(40, 50);
    // Tweak the counter to be less than the start counter of the first revertible call.
    builder.split_calls(59);
    builder.add_private_call_request(60, 70);

    builder.validate();
}

#[test(should_fail_with="min_revertible_side_effect_counter must be greater than the end counter of the last non revertible item")]
fn validate_private_call_requests_split_private_calls_less_than_last_non_revertible_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new_first_call();

    builder.add_private_call_request(20, 30);
    builder.add_private_call_request(40, 50);
    // Tweak the counter to be less than the end counter of the last non-revertible call.
    builder.split_calls(49);
    builder.add_private_call_request(60, 70);

    builder.validate();
}

#[test(should_fail_with="min_revertible_side_effect_counter must be greater than the end counter of the last non revertible item")]
fn validate_private_call_requests_split_private_calls_equal_last_non_revertible_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new_first_call();

    builder.add_private_call_request(20, 30);
    builder.add_private_call_request(40, 50);
    // Tweak the counter to equal the end counter of the last non-revertible call.
    builder.split_calls(50);

    builder.validate();
}

#[test]
fn validate_private_call_requests_split_private_calls_0_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new_first_call();

    // Set the counter to be 0.
    builder.split_calls(0);
    builder.add_private_call_request(20, 30);
    builder.add_private_call_request(40, 50);

    builder.validate();
}
