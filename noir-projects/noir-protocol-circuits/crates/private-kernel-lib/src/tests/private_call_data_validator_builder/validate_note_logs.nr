use crate::tests::private_call_data_validator_builder::PrivateCallDataValidatorBuilder;
use dep::types::{abis::note_hash::NoteHash, address::AztecAddress};

#[test]
fn validate_note_logs_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_note_hashes_with_logs(2);

    builder.validate();
}

#[test(should_fail_with="could not find note hash linked to note log")]
fn validate_note_logs_random_note_hash_counter_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_note_hashes_with_logs(2);
    // Tweak the note_hash_counter to not match any note hash's counter.
    builder.private_call.note_encrypted_logs_hashes.storage[1].note_hash_counter += 100;

    builder.validate();
}

#[test(should_fail_with="could not link a note log to a note hash in another contract")]
fn validate_note_logs_zero_note_hash_counter_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_note_hashes_with_logs(2);
    // Tweak the note_hash_counter to be 0.
    builder.private_call.note_encrypted_logs_hashes.storage[1].note_hash_counter = 0;

    builder.validate();
}

#[test(should_fail_with="could not link a note log to a note hash in another contract")]
fn validate_note_logs_mismatch_contract_address_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new_from_counter(50);

    // Create a note hash emitted from a different contract.
    let another_contract_address = AztecAddress::from_field(9999);
    let previous_note_hash = NoteHash { value: 1, counter: 17 }.scope(another_contract_address);
    builder.previous_note_hashes.push(previous_note_hash);

    // Add a not log linked to the previous note hash.
    builder.private_call.add_note_encrypted_log_hash(123, 2, previous_note_hash.counter());

    builder.validate();
}
