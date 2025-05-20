use crate::tests::private_call_data_validator_builder::PrivateCallDataValidatorBuilder;

/**
 * Private call.
 */
#[test]
fn validate_counters_private_call_succeeds() {
    let builder = PrivateCallDataValidatorBuilder::new_from_counter(23);
    builder.validate();
}

#[test]
fn validate_counters_private_call_from_0_counter_succeeds() {
    let builder = PrivateCallDataValidatorBuilder::new_from_counter(0);
    builder.validate();
}

#[test(should_fail_with = "private call has incorrect counter range")]
fn validate_counters_private_call_no_counter_range_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.counter = builder.private_call.counter_start;

    builder.validate();
}

#[test(should_fail_with = "private call has incorrect counter range")]
fn validate_counters_private_call_negative_call_counter_range_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.counter = builder.private_call.counter_start - 1;

    builder.validate();
}

/**
 * Note hashes
 */
#[test]
fn validate_counters_note_hashes_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_note_hashes(2);

    builder.validate();
}

#[test(should_fail_with = "counter must be larger than the counter of the previous item")]
fn validate_counters_note_hash_counter_same_as_call_counter_start_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_note_hashes(1);
    // Tweak the counter of the first note hash to EQUAL the start counter of the call.
    let mut note_hash = builder.private_call.note_hashes.get(0);
    note_hash.note_hash.counter = builder.private_call.counter_start;
    builder.private_call.note_hashes.set(0, note_hash);

    builder.validate();
}

#[test(should_fail_with = "counter must be larger than the counter of the previous item")]
fn validate_counters_note_hash_counter_smaller_than_call_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_note_hashes(1);
    // Tweak the counter of the first note hash to be LESS than the start counter of the call.
    let mut note_hash = builder.private_call.note_hashes.get(0);
    note_hash.note_hash.counter = builder.private_call.counter_start - 1;
    builder.private_call.note_hashes.set(0, note_hash);

    builder.validate();
}

#[test(should_fail_with = "counter must be larger than the counter of the previous item")]
fn validate_counters_note_hash_identical_counters_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_note_hashes(2);
    let counter_start = builder.private_call.counter_start;
    // Tweak the counter of the second note hash to EQUAL the counter of the first note hash.
    let mut note_hash_0 = builder.private_call.note_hashes.get(0);
    note_hash_0.note_hash.counter = counter_start + 1;
    builder.private_call.note_hashes.set(0, note_hash_0);
    let mut note_hash_1 = builder.private_call.note_hashes.get(1);
    note_hash_1.note_hash.counter = counter_start + 1;
    builder.private_call.note_hashes.set(1, note_hash_1);

    builder.validate();
}

#[test(should_fail_with = "counter must be larger than the counter of the previous item")]
fn validate_counters_note_hash_unordered_counters_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_note_hashes(2);
    let counter_start = builder.private_call.counter_start;
    // Tweak the counter of the second note hash to be LESS than the counter of the first note hash.
    let mut note_hash_0 = builder.private_call.note_hashes.get(0);
    note_hash_0.note_hash.counter = counter_start + 2;
    builder.private_call.note_hashes.set(0, note_hash_0);
    let mut note_hash_1 = builder.private_call.note_hashes.get(1);
    note_hash_1.note_hash.counter = counter_start + 1;
    builder.private_call.note_hashes.set(1, note_hash_1);

    builder.validate();
}

#[test(should_fail_with = "counter must be smaller than the end counter of the call")]
fn validate_counters_note_hash_counter_larger_than_call_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_note_hashes(2);
    // Tweak the counter of the second note hash to be GREATER than the end counter of the call.
    let mut note_hash = builder.private_call.note_hashes.get(1);
    note_hash.note_hash.counter = builder.private_call.counter + 1;
    builder.private_call.note_hashes.set(1, note_hash);

    builder.validate();
}

#[test(should_fail_with = "counter must be smaller than the end counter of the call")]
fn validate_counters_note_hash_counter_same_as_call_counter_end_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_note_hashes(2);
    // Tweak the counter of the second note hash to EQUAL the end counter of the call.
    let mut note_hash = builder.private_call.note_hashes.get(1);
    note_hash.note_hash.counter = builder.private_call.counter;
    builder.private_call.note_hashes.set(1, note_hash);

    builder.validate();
}

/**
 * Private call requests.
 */
#[test]
fn validate_counters_private_call_requests_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_private_call_requests(2);

    builder.validate();
}

#[test(should_fail_with = "start counter must be larger than the end counter of the previous call")]
fn validate_counters_private_call_requests_less_than_call_start_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_private_call_requests(1);
    // Tweak the start counter of the first nested call to be LESS than the start counter of the call.
    let mut call_request = builder.private_call.private_call_requests.get(0);
    call_request.start_side_effect_counter = builder.private_call.counter_start - 1;
    builder.private_call.private_call_requests.set(0, call_request);

    builder.validate();
}

#[test(should_fail_with = "start counter must be larger than the end counter of the previous call")]
fn validate_counters_private_call_requests_equal_call_start_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_private_call_requests(1);
    // Tweak the start counter of the call to EQUAL the start counter of the first nested call.
    let mut call_request = builder.private_call.private_call_requests.get(0);
    call_request.start_side_effect_counter = builder.private_call.counter_start;
    builder.private_call.private_call_requests.set(0, call_request);

    builder.validate();
}

#[test(should_fail_with = "start counter must be larger than the end counter of the previous call")]
fn validate_counters_private_call_requests_less_than_previous_end_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_private_call_requests(2);
    // Tweak the start counter of the second nested call to be LESS than the end counter of the first nested call.
    let counter_end = builder.private_call.private_call_requests.get(0).end_side_effect_counter;
    let mut call_request = builder.private_call.private_call_requests.get(1);
    call_request.start_side_effect_counter = counter_end - 1;
    builder.private_call.private_call_requests.set(1, call_request);

    builder.validate();
}

#[test(should_fail_with = "start counter must be larger than the end counter of the previous call")]
fn validate_counters_private_call_requests_same_as_previous_end_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_private_call_requests(2);
    // Tweak the start counter of the second nested call to EQUAL the end counter of the first nested call.
    let counter_end = builder.private_call.private_call_requests.get(0).end_side_effect_counter;
    let mut call_request = builder.private_call.private_call_requests.get(1);
    call_request.start_side_effect_counter = counter_end;
    builder.private_call.private_call_requests.set(1, call_request);

    builder.validate();
}

#[test(should_fail_with = "nested call has incorrect counter range")]
fn validate_counters_private_call_requests_end_less_than_start_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_private_call_requests(1);
    // Tweak the end counter of the first nested call to be LESS than its start counter.
    let mut call_request = builder.private_call.private_call_requests.get(0);
    call_request.end_side_effect_counter = call_request.start_side_effect_counter - 1;
    builder.private_call.private_call_requests.set(0, call_request);

    builder.validate();
}

#[test(should_fail_with = "nested call has incorrect counter range")]
fn validate_counters_private_call_requests_end_equal_start_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_private_call_requests(1);
    // Tweak the end counter of the first nested call to EQUAL its start counter.
    let mut call_request = builder.private_call.private_call_requests.get(0);
    call_request.end_side_effect_counter = call_request.start_side_effect_counter;
    builder.private_call.private_call_requests.set(0, call_request);

    builder.validate();
}

#[test(should_fail_with = "end counter must be smaller than the end counter of the parent call")]
fn validate_counters_private_call_requests_greater_than_call_end_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_private_call_requests(1);
    // Tweak the end counter of the nested call to be GREATER than the end counter of the call.
    let counter_end = builder.private_call.counter;
    let mut call_request = builder.private_call.private_call_requests.get(0);
    call_request.end_side_effect_counter = counter_end + 1;
    builder.private_call.private_call_requests.set(0, call_request);

    builder.validate();
}

#[test(should_fail_with = "end counter must be smaller than the end counter of the parent call")]
fn validate_counters_private_call_requests_equal_call_end_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_private_call_requests(1);
    // Tweak the end counter of the nested call to EQUAL the end counter of the call.
    let mut call_request = builder.private_call.private_call_requests.get(0);
    call_request.end_side_effect_counter = builder.private_call.counter;
    builder.private_call.private_call_requests.set(0, call_request);

    builder.validate();
}
