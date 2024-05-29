use crate::tests::private_call_data_validator_builder::PrivateCallDataValidatorBuilder;
use dep::types::constants::MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL;

impl PrivateCallDataValidatorBuilder {
    pub fn new_first_call() -> Self {
        PrivateCallDataValidatorBuilder::new_from_counter(0)
    }

    pub fn split_calls(&mut self, counter: u32) {
        self.private_call.min_revertible_side_effect_counter = counter;
        self.first_revertible_private_call_request_index = self.private_call.private_call_requests.len();
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
fn validate_as_first_call_regular_call_succeeds() {
    let builder = PrivateCallDataValidatorBuilder::new();
    builder.validate_as_first_call();
}

#[test(should_fail_with="Users cannot make a static call")]
fn validate_as_first_call_static_call_fails() {
    let builder = PrivateCallDataValidatorBuilder::new().is_static_call();
    builder.validate_as_first_call();
}

#[test(should_fail_with="Users cannot make a delegatecall")]
fn validate_as_first_call_delegate_call_fails() {
    let builder = PrivateCallDataValidatorBuilder::new().is_delegate_call();
    builder.validate_as_first_call();
}

/**
 * Splitting call requests.
 */

#[test]
fn validate_as_first_call_split_private_calls_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new_first_call();

    builder.add_private_call_request(20, 30);
    builder.add_private_call_request(40, 50);
    builder.split_calls(60);
    builder.add_private_call_request(60, 70);

    builder.validate_as_first_call();
}

#[test]
fn validate_as_first_call_split_private_empty_revertible_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new_first_call();

    builder.add_private_call_request(20, 30);
    builder.add_private_call_request(40, 50);
    builder.split_calls(51);

    builder.validate_as_first_call();
}

#[test]
fn validate_as_first_call_split_private_empty_non_revertible_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new_first_call();

    builder.split_calls(20);
    builder.add_private_call_request(20, 30);
    builder.add_private_call_request(40, 50);

    builder.validate_as_first_call();
}

#[test]
fn validate_as_first_call_split_private_full_non_revertible_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new_first_call();

    builder.private_call.append_private_call_requests(MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL);
    builder.split_calls(builder.private_call.counter);

    builder.validate_as_first_call();
}

#[test]
fn validate_as_first_call_split_private_calls_less_than_first_revertible_success() {
    let mut builder = PrivateCallDataValidatorBuilder::new_first_call();

    builder.add_private_call_request(20, 30);
    builder.add_private_call_request(40, 50);
    // Tweak the counter to be less than the start counter of the first revertible call.
    builder.split_calls(59);
    builder.add_private_call_request(60, 70);

    builder.validate_as_first_call();
}

#[test(should_fail_with="min_revertible_side_effect_counter must be greater than the end counter of the last non revertible item")]
fn validate_as_first_call_split_private_calls_less_than_last_non_revertible_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new_first_call();

    builder.add_private_call_request(20, 30);
    builder.add_private_call_request(40, 50);
    // Tweak the counter to be less than the end counter of the last non-revertible call.
    builder.split_calls(49);
    builder.add_private_call_request(60, 70);

    builder.validate_as_first_call();
}

#[test(should_fail_with="min_revertible_side_effect_counter must be greater than the end counter of the last non revertible item")]
fn validate_as_first_call_split_private_calls_equal_last_non_revertible_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new_first_call();

    builder.add_private_call_request(20, 30);
    builder.add_private_call_request(40, 50);
    // Tweak the counter to equal the end counter of the last non-revertible call.
    builder.split_calls(50);

    builder.validate_as_first_call();
}

#[test(should_fail_with="min_revertible_side_effect_counter must be less than or equal to the start counter of the first revertible item")]
fn validate_as_first_call_split_private_calls_greater_than_first_revertible_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new_first_call();

    builder.add_private_call_request(20, 30);
    builder.add_private_call_request(40, 50);
    // Tweak the counter to be greater than the start counter of the first revertible call.
    builder.split_calls(61);
    builder.add_private_call_request(60, 70);

    builder.validate_as_first_call();
}

#[test]
fn validate_as_first_call_split_private_calls_0_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new_first_call();

    // Set the counter to be 0.
    builder.split_calls(0);
    builder.add_private_call_request(20, 30);
    builder.add_private_call_request(40, 50);

    builder.validate_as_first_call();
}

#[test(should_fail_with="min_revertible_side_effect_counter must be greater than the end counter of the last non revertible item")]
fn validate_as_first_call_split_private_calls_0_wrong_hint_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new_first_call();

    builder.split_calls(0);
    // Set the index hint to be 1.
    builder.first_revertible_private_call_request_index = 1;
    builder.add_private_call_request(20, 30);
    builder.add_private_call_request(40, 50);

    builder.validate_as_first_call();
}

#[test(should_fail_with="min_revertible_side_effect_counter must be less than or equal to the start counter of the first revertible item")]
fn validate_as_first_call_split_private_calls_index_hint_greater_than_len_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new_first_call();

    builder.add_private_call_request(20, 30);
    builder.add_private_call_request(40, 50);
    builder.split_calls(51);
    // Increase the index by 1.
    builder.first_revertible_private_call_request_index += 1;

    builder.validate_as_first_call();
}
