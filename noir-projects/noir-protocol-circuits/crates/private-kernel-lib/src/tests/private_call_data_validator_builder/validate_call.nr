use crate::tests::private_call_data_validator_builder::PrivateCallDataValidatorBuilder;

#[test]
fn validate_call_is_regular_succeeds() {
    let builder = PrivateCallDataValidatorBuilder::new();
    builder.validate();
}

#[test(should_fail_with="call stack storage address does not match expected contract address")]
fn validate_call_is_regular_mismatch_storage_contract_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    // Change the storage contract address to be a different value.
    builder.private_call.storage_contract_address.inner += 1;

    builder.validate();
}

#[test]
fn validate_call_is_delegate_succeeds() {
    let builder = PrivateCallDataValidatorBuilder::new().is_delegate_call();
    builder.validate();
}

#[test]
fn validate_call_is_delegate_static_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_delegate_call().is_static_call();
    builder.validate();
}

#[test(should_fail_with="current contract address must not match storage contract address for delegate calls")]
fn validate_call_is_delegate_call_from_same_contract_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_delegate_call();

    // Change the caller's storage contract address to be the same as the contract address.
    builder.private_call.storage_contract_address = builder.private_call.contract_address;

    builder.validate();
}

#[test]
fn validate_call_is_static_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_static_call();
    builder.validate();
}

#[test(should_fail_with="call stack storage address does not match expected contract address")]
fn validate_call_is_static_mismatch_storage_contract_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_static_call();

    // Change the storage contract address to be a different value.
    builder.private_call.storage_contract_address.inner += 1;

    builder.validate();
}

#[test(should_fail_with="note_hashes must be empty for static calls")]
fn validate_call_is_static_creating_note_hashes_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_static_call();

    builder.private_call.append_note_hashes(1);

    builder.validate();
}

#[test(should_fail_with="nullifiers must be empty for static calls")]
fn validate_call_is_static_creating_nullifiers_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_static_call();

    builder.private_call.append_nullifiers(1);

    builder.validate();
}

#[test(should_fail_with="l2_to_l1_msgs must be empty for static calls")]
fn validate_call_is_static_creating_l2_to_l1_msgs_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_static_call();

    builder.private_call.append_l2_to_l1_msgs(1);

    builder.validate();
}

#[test(should_fail_with="note_encrypted_logs_hashes must be empty for static calls")]
fn validate_call_is_static_creating_note_encrypted_logs_hashes_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_static_call();

    builder.private_call.append_note_encrypted_log_hashes(1);

    builder.validate();
}

#[test(should_fail_with="encrypted_logs_hashes must be empty for static calls")]
fn validate_call_is_static_creating_encrypted_logs_hashes_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_static_call();

    builder.private_call.append_encrypted_log_hashes(1);

    builder.validate();
}

#[test(should_fail_with="unencrypted_logs_hashes must be empty for static calls")]
fn validate_call_is_static_creating_unencrypted_logs_hashes_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_static_call();

    builder.private_call.append_unencrypted_log_hashes(1);

    builder.validate();
}
