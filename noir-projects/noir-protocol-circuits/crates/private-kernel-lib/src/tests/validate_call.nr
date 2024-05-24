use crate::tests::private_call_data_validator_builder::PrivateCallDataValidatorBuilder;
use dep::types::{
    abis::{note_hash::NoteHash, nullifier::Nullifier, log_hash::{LogHash, EncryptedLogHash}},
    address::EthAddress, messaging::l2_to_l1_message::L2ToL1Message
};

#[test]
fn validate_call_is_regular_succeeds() {
    let builder = PrivateCallDataValidatorBuilder::new();
    builder.validate();
}

#[test(should_fail_with="call stack storage address does not match expected contract address")]
fn validate_call_is_regular_mismatch_storage_contract_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    // Change the storage contract address to be a different value.
    builder.private_call.public_inputs.call_context.storage_contract_address.inner += 1;

    builder.validate();
}

#[test]
fn validate_call_is_delegate_succeeds() {
    let builder = PrivateCallDataValidatorBuilder::new().is_delegate_call();
    builder.validate();
}

#[test(should_fail_with="current contract address must not match storage contract address for delegate calls")]
fn validate_call_is_delegate_call_from_same_contract_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_delegate_call();

    // Change the caller's storage contract address to be the same as the contract address.
    builder.private_call.public_inputs.call_context.storage_contract_address = builder.private_call.contract_address;

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
    builder.private_call.public_inputs.call_context.storage_contract_address.inner += 1;

    builder.validate();
}

#[test(should_fail_with="new_note_hashes must be empty for static calls")]
fn validate_call_is_static_creating_note_hashes_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_static_call();

    builder.private_call.public_inputs.new_note_hashes.push(NoteHash { value: 1, counter: 0 });

    builder.validate();
}

#[test(should_fail_with="new_nullifiers must be empty for static calls")]
fn validate_call_is_static_creating_nullifiers_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_static_call();

    builder.private_call.public_inputs.new_nullifiers.push(Nullifier { value: 1, counter: 0, note_hash: 0 });

    builder.validate();
}

#[test(should_fail_with="new_l2_to_l1_msgs must be empty for static calls")]
fn validate_call_is_static_creating_l2_to_l1_msgs_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_static_call();

    builder.private_call.public_inputs.new_l2_to_l1_msgs.push(L2ToL1Message { recipient: EthAddress::from_field(6), content: 9123, counter: 0 });

    builder.validate();
}

#[test(should_fail_with="encrypted_logs_hashes must be empty for static calls")]
fn validate_call_is_static_creating_encrypted_logs_hashes_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_static_call();

    builder.private_call.public_inputs.encrypted_logs_hashes.push(EncryptedLogHash { value: 9123, counter: 1, length: 2, randomness: 4 });

    builder.validate();
}

#[test(should_fail_with="unencrypted_logs_hashes must be empty for static calls")]
fn validate_call_is_static_creating_unencrypted_logs_hashes_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new().is_static_call();

    builder.private_call.public_inputs.unencrypted_logs_hashes.push(LogHash { value: 9123, counter: 1, length: 2 });

    builder.validate();
}
