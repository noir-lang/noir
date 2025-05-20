use crate::tests::private_call_data_validator_builder::PrivateCallDataValidatorBuilder;
use dep::types::{
    abis::{note_hash::NoteHash, side_effect::Ordered},
    address::AztecAddress,
    traits::FromField,
};

#[test]
fn validate_note_logs_succeeds() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_note_hashes_with_logs(2);

    builder.validate();
}

#[test(should_fail_with = "could not find note hash linked to note log")]
fn validate_note_logs_random_note_hash_counter_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.append_note_hashes_with_logs(2);
    // Tweak the note_hash_counter to not match any note hash's counter.
    let mut note_hash = builder.private_call.private_logs.get(1);
    note_hash.inner.note_hash_counter += 100;
    builder.private_call.private_logs.set(1, note_hash);

    builder.validate();
}

#[test(should_fail_with = "could not link a note log to a note hash in another contract")]
fn validate_note_logs_mismatch_contract_address_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new_from_counter(50);

    // Create a note hash emitted from a different contract.
    let another_contract_address = AztecAddress::from_field(9999);
    let previous_note_hash = NoteHash { value: 1, counter: 17 }.scope(another_contract_address);
    builder.previous_note_hashes.push(previous_note_hash);

    // Add a note log linked to the previous note hash.
    builder.private_call.append_private_logs_for_note(1, previous_note_hash.counter());

    builder.validate();
}
