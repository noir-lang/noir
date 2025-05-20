use crate::tests::private_call_data_validator_builder::PrivateCallDataValidatorBuilder;
use dep::types::traits::Empty;

fn unshift_empty_item<T, let N: u32>(vec: &mut BoundedVec<T, N>)
where
    T: Empty,
{
    let empty_item = T::empty();
    let first_item = vec.get(0);
    vec.push(first_item);
    vec.set(0, empty_item);
}

#[test(should_fail_with = "invalid array")]
fn validate_arrays_malformed_note_hash_read_requests_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_note_hash_read_requests(1);
    unshift_empty_item(&mut builder.private_call.note_hash_read_requests);

    builder.validate();
}

#[test(should_fail_with = "invalid array")]
fn validate_arrays_malformed_nullifier_read_requests_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_nullifier_read_requests(1);
    unshift_empty_item(&mut builder.private_call.nullifier_read_requests);

    builder.validate();
}

#[test(should_fail_with = "invalid array")]
fn validate_arrays_malformed_key_validation_requests_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_key_validation_requests(1);
    unshift_empty_item(
        &mut builder.private_call.scoped_key_validation_requests_and_generators,
    );

    builder.validate();
}

#[test(should_fail_with = "invalid array")]
fn validate_arrays_malformed_note_hashes_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_note_hashes(1);
    unshift_empty_item(&mut builder.private_call.note_hashes);

    builder.validate();
}

#[test(should_fail_with = "invalid array")]
fn validate_arrays_malformed_nullifiers_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_nullifiers(1);
    unshift_empty_item(&mut builder.private_call.nullifiers);

    builder.validate();
}

#[test(should_fail_with = "invalid array")]
fn validate_arrays_malformed_l2_to_l1_msgs_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_l2_to_l1_msgs(1);
    unshift_empty_item(&mut builder.private_call.l2_to_l1_msgs);

    builder.validate();
}

#[test(should_fail_with = "invalid array")]
fn validate_arrays_malformed_private_call_requests_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_private_call_requests(1);
    unshift_empty_item(&mut builder.private_call.private_call_requests);

    builder.validate();
}

#[test(should_fail_with = "invalid array")]
fn validate_arrays_malformed_public_call_stack_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_public_call_requests(1);
    unshift_empty_item(&mut builder.private_call.public_call_requests);

    builder.validate();
}

#[test(should_fail_with = "invalid array")]
fn validate_arrays_malformed_private_logs() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_private_logs(1);
    unshift_empty_item(&mut builder.private_call.private_logs);

    builder.validate();
}
