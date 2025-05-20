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
        let mut call_request = self.private_call.private_call_requests.get(index);
        call_request.start_side_effect_counter = counter_start;
        call_request.end_side_effect_counter = counter_end;
        self.private_call.private_call_requests.set(index, call_request);
        self.private_call.counter = counter_end + 1;
    }
}

#[test]
fn validate_private_call_requests_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_private_call_requests(2);

    builder.validate();
}

#[test]
fn validate_private_call_requests_from_static_call_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_static_call();

    builder.private_call.append_private_call_requests(2);

    builder.validate();
}

#[test(should_fail_with = "incorrect msg_sender for call request")]
fn validate_private_call_requests_incorrect_msg_sender_for_regular_call_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_private_call_requests(1);
    // Change the msg_sender to be the caller's msg_sender.
    let mut call_request = builder.private_call.private_call_requests.get(0);
    call_request.call_context.msg_sender = builder.private_call.msg_sender;
    builder.private_call.private_call_requests.set(0, call_request);

    builder.validate();
}

#[test(should_fail_with = "static call cannot make non-static calls")]
fn validate_private_call_requests_static_call_regular_call_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_static_call();

    builder.private_call.append_private_call_requests(1);
    // Tweak the request to be making a non-static call.
    let mut call_request = builder.private_call.private_call_requests.get(0);
    call_request.call_context.is_static_call = false;
    builder.private_call.private_call_requests.set(0, call_request);

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

#[test(should_fail_with = "min_revertible_side_effect_counter must be greater than the end counter of the last non revertible item")]
fn validate_private_call_requests_split_private_calls_less_than_last_non_revertible_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new_first_call();

    builder.add_private_call_request(20, 30);
    builder.add_private_call_request(40, 50);
    // Tweak the counter to be less than the end counter of the last non-revertible call.
    builder.split_calls(49);
    builder.add_private_call_request(60, 70);

    builder.validate();
}

#[test(should_fail_with = "min_revertible_side_effect_counter must be greater than the end counter of the last non revertible item")]
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
