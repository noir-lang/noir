mod transient_data_reset_hints;

use dep::types::{
    abis::{note_hash::ScopedNoteHash, nullifier::ScopedNullifier, log_hash::NoteLogHash},
    traits::is_empty
};

pub fn verify_squashed_transient_data<NUM_NOTE_HASHES, NUM_NULLIFIERS, NUM_LOGS>(
    note_hashes: [ScopedNoteHash; NUM_NOTE_HASHES],
    nullifiers: [ScopedNullifier; NUM_NULLIFIERS],
    note_logs: [NoteLogHash; NUM_LOGS],
    expected_note_hashes: [ScopedNoteHash; NUM_NOTE_HASHES],
    expected_nullifiers: [ScopedNullifier; NUM_NULLIFIERS],
    expected_note_logs: [NoteLogHash; NUM_LOGS],
    transient_nullifier_indexes_for_note_hashes: [u32; NUM_NOTE_HASHES],
    transient_note_hash_indexes_for_nullifiers: [u32; NUM_NULLIFIERS],
    transient_or_propagated_note_hash_indexes_for_logs: [u32; NUM_LOGS]
) {
    let mut note_hashes_kept = 0;
    let mut note_hashes_removed = 0;
    for i in 0..NUM_NOTE_HASHES {
        let note_hash = note_hashes[i];
        let nullifier_index = transient_nullifier_indexes_for_note_hashes[i];
        if nullifier_index == NUM_NULLIFIERS { // The note hash has no corresponding nullifier.
            assert_eq(expected_note_hashes[note_hashes_kept], note_hash, "Propagated note hash does not match");
            note_hashes_kept += 1;
        } else {
            let nullifier = nullifiers[nullifier_index];
            assert_eq(
                note_hash.value(), nullifier.nullified_note_hash(), "Value of the hinted transient note hash does not match"
            );
            assert_eq(
                note_hash.contract_address, nullifier.contract_address, "Contract address of the hinted transient note hash does not match"
            );

            note_hashes_removed += 1;

            // For each note hash removed, an empty item is padded to the right.
            let padded_note_hash = expected_note_hashes[NUM_NOTE_HASHES - note_hashes_removed];
            assert(is_empty(padded_note_hash), "Empty note hash must be padded to the right");
        }
    }

    let mut nullifiers_kept = 0;
    let mut nullifiers_removed = 0;
    for i in 0..NUM_NULLIFIERS {
        let nullifier = nullifiers[i];
        let index_hint = transient_note_hash_indexes_for_nullifiers[i];
        if index_hint == NUM_NOTE_HASHES {
            assert_eq(expected_nullifiers[nullifiers_kept], nullifier, "Propagated nullifier does not match");
            nullifiers_kept += 1;
        } else {
            let transient_nullifier_index = transient_nullifier_indexes_for_note_hashes[index_hint];
            assert_eq(transient_nullifier_index, i, "Invalid transient nullifier index hint");

            nullifiers_removed += 1;

            // For each nullifier removed, an empty item is padded to the right.
            let padded_nullifier = expected_nullifiers[NUM_NULLIFIERS - nullifiers_removed];
            assert(is_empty(padded_nullifier), "Empty nullifier must be padded to the right");
        }
    }

    let mut note_logs_kept = 0;
    let mut note_logs_removed = 0;
    for i in 0..NUM_LOGS {
        let log = note_logs[i];
        let index_hint = transient_or_propagated_note_hash_indexes_for_logs[i];
        // - For non-empty logs, their note_hash_counter can't be 0. It must match the counter of a note hash, which is verified in private_call_data_validator.
        //   Therefor, is_propagated will be false if index_hint incorrectly points to an empty expected_note_hash and fails the check inside the else statement.
        // - For empty logs, we expect is_propagated to be true, making them part of the note_logs_kept.
        //   index_hint might point them to the same empty note. This doesn't matter because we increment the value of 
        //   note_logs_kept and ensure that the correct number of empty logs are present in the expected_note_logs.
        let is_propagated = expected_note_hashes[index_hint].counter() == log.note_hash_counter;
        if is_propagated {
            assert_eq(expected_note_logs[note_logs_kept], log, "Propagated note log does not match");
            note_logs_kept += 1;
        } else {
            let transient_note_hash = note_hashes[index_hint];
            assert_eq(
                log.note_hash_counter, transient_note_hash.counter(), "Value of the hinted transient note hash does not match log"
            );
            // We may have many logs for each nullified note, so we do the below checks per log,
            // rather than per note
            let transient_nullifier_index = transient_nullifier_indexes_for_note_hashes[index_hint];
            assert(transient_nullifier_index != NUM_NULLIFIERS, "Hinted transient note log not nullified");

            note_logs_removed += 1;

            // For each log removed, an empty item is padded to the right.
            let padded_log = expected_note_logs[NUM_LOGS - note_logs_removed];
            assert(is_empty(padded_log), "Empty log must be padded to the right");
        }
    }

    assert_eq(
        note_hashes_removed, nullifiers_removed, "Inconsistent number of note hashes and nullifiers removed"
    );
}

mod tests {
    use crate::reset::transient_data::verify_squashed_transient_data;
    use dep::types::{
        abis::{note_hash::{NoteHash, ScopedNoteHash}, nullifier::{Nullifier, ScopedNullifier}, log_hash::NoteLogHash},
        address::AztecAddress
    };

    global contract_address = AztecAddress::from_field(987654);

    struct TestDataBuilder<NUM_NOTE_HASHES, NUM_NULLIFIERS, NUM_LOGS> {
        note_hashes: [ScopedNoteHash; NUM_NOTE_HASHES],
        nullifiers: [ScopedNullifier; NUM_NULLIFIERS],
        note_logs: [NoteLogHash; NUM_LOGS],
        expected_note_hashes: [ScopedNoteHash; NUM_NOTE_HASHES],
        expected_nullifiers: [ScopedNullifier; NUM_NULLIFIERS],
        expected_note_logs: [NoteLogHash; NUM_LOGS],
        transient_nullifier_indexes_for_note_hashes: [u32; NUM_NOTE_HASHES],
        transient_note_hash_indexes_for_nullifiers: [u32; NUM_NULLIFIERS],
        transient_or_propagated_note_hash_indexes_for_logs: [u32; NUM_LOGS]
    }

    impl<NUM_NOTE_HASHES, NUM_NULLIFIERS, NUM_LOGS> TestDataBuilder<NUM_NOTE_HASHES, NUM_NULLIFIERS, NUM_LOGS> {
        pub fn new() -> TestDataBuilder<5, 4, 3> {
            let note_hashes = [
                NoteHash { value: 11, counter: 100 }.scope(contract_address),
                NoteHash { value: 22, counter: 200 }.scope(contract_address),
                NoteHash { value: 33, counter: 300 }.scope(contract_address),
                ScopedNoteHash::empty(),
                ScopedNoteHash::empty()
            ];

            let nullifiers = [
                Nullifier { value: 44, counter: 400, note_hash: 33 }.scope(contract_address),
                Nullifier { value: 55, counter: 500, note_hash: 11 }.scope(contract_address),
                Nullifier { value: 66, counter: 600, note_hash: 0 }.scope(contract_address),
                ScopedNullifier::empty()
            ];

            let note_logs = [
                NoteLogHash { value: 77, counter: 700, length: 70, note_hash_counter: 100 },
                NoteLogHash { value: 88, counter: 800, length: 80, note_hash_counter: 200 },
                NoteLogHash::empty()
            ];

            let mut expected_note_hashes = [ScopedNoteHash::empty(); 5];
            expected_note_hashes[0] = note_hashes[1];
            let mut expected_nullifiers = [ScopedNullifier::empty(); 4];
            expected_nullifiers[0] = nullifiers[2];
            let mut expected_note_logs = [NoteLogHash::empty(); 3];
            expected_note_logs[0] = note_logs[1];

            let transient_nullifier_indexes_for_note_hashes = [1, 4, 0, 4, 4];
            let transient_note_hash_indexes_for_nullifiers = [2, 0, 5, 5];
            let transient_or_propagated_note_hash_indexes_for_logs = [0, 0, 1];

            TestDataBuilder {
                note_hashes,
                nullifiers,
                note_logs,
                expected_note_hashes,
                expected_nullifiers,
                expected_note_logs,
                transient_nullifier_indexes_for_note_hashes,
                transient_note_hash_indexes_for_nullifiers,
                transient_or_propagated_note_hash_indexes_for_logs
            }
        }

        pub fn new_clear_all() -> TestDataBuilder<3, 3, 4> {
            let note_hashes = [
                NoteHash { value: 11, counter: 100 }.scope(contract_address),
                NoteHash { value: 22, counter: 200 }.scope(contract_address),
                NoteHash { value: 33, counter: 300 }.scope(contract_address)
            ];

            let nullifiers = [
                Nullifier { value: 44, counter: 400, note_hash: 33 }.scope(contract_address),
                Nullifier { value: 55, counter: 500, note_hash: 11 }.scope(contract_address),
                Nullifier { value: 66, counter: 600, note_hash: 22 }.scope(contract_address)
            ];

            // tests removing two logs for one note hash
            let note_logs = [
                NoteLogHash { value: 77, counter: 700, length: 70, note_hash_counter: 100 },
                NoteLogHash { value: 88, counter: 800, length: 80, note_hash_counter: 300 },
                NoteLogHash { value: 99, counter: 900, length: 90, note_hash_counter: 200 },
                NoteLogHash { value: 111, counter: 1000, length: 100, note_hash_counter: 300 }
            ];

            let expected_note_hashes = [ScopedNoteHash::empty(); 3];
            let expected_nullifiers = [ScopedNullifier::empty(); 3];
            let expected_note_logs = [NoteLogHash::empty(); 4];

            let transient_nullifier_indexes_for_note_hashes = [1, 2, 0];
            let transient_note_hash_indexes_for_nullifiers = [2, 0, 1];
            let transient_or_propagated_note_hash_indexes_for_logs = [0, 2, 1, 2];

            TestDataBuilder {
                note_hashes,
                nullifiers,
                note_logs,
                expected_note_hashes,
                expected_nullifiers,
                expected_note_logs,
                transient_nullifier_indexes_for_note_hashes,
                transient_note_hash_indexes_for_nullifiers,
                transient_or_propagated_note_hash_indexes_for_logs
            }
        }

        pub fn new_identical_note_hashes() -> TestDataBuilder<3, 3, 4> {
            let note_hashes = [
                NoteHash { value: 11, counter: 100 }.scope(contract_address),
                NoteHash { value: 11, counter: 200 }.scope(contract_address),
                NoteHash { value: 11, counter: 600 }.scope(contract_address)
            ];

            let nullifiers = [
                Nullifier { value: 33, counter: 300, note_hash: 11 }.scope(contract_address),
                Nullifier { value: 44, counter: 400, note_hash: 0 }.scope(contract_address),
                Nullifier { value: 55, counter: 500, note_hash: 11 }.scope(contract_address)
            ];

            let note_logs = [
                NoteLogHash { value: 77, counter: 701, length: 70, note_hash_counter: 200 },
                NoteLogHash { value: 77, counter: 702, length: 70, note_hash_counter: 200 },
                NoteLogHash { value: 77, counter: 703, length: 70, note_hash_counter: 200 },
                NoteLogHash { value: 88, counter: 800, length: 80, note_hash_counter: 600 }
            ];

            let expected_note_hashes = [note_hashes[2], ScopedNoteHash::empty(), ScopedNoteHash::empty()];
            let expected_nullifiers = [nullifiers[1], ScopedNullifier::empty(), ScopedNullifier::empty()];
            let expected_note_logs = [note_logs[3], NoteLogHash::empty(), NoteLogHash::empty(), NoteLogHash::empty()];

            let transient_nullifier_indexes_for_note_hashes = [0, 2, 3];
            let transient_note_hash_indexes_for_nullifiers = [0, 3, 1];
            let transient_or_propagated_note_hash_indexes_for_logs = [1, 1, 1, 0];

            TestDataBuilder {
                note_hashes,
                nullifiers,
                note_logs,
                expected_note_hashes,
                expected_nullifiers,
                expected_note_logs,
                transient_nullifier_indexes_for_note_hashes,
                transient_note_hash_indexes_for_nullifiers,
                transient_or_propagated_note_hash_indexes_for_logs
            }
        }

        pub fn verify(self) {
            let _void = verify_squashed_transient_data(
                self.note_hashes,
                self.nullifiers,
                self.note_logs,
                self.expected_note_hashes,
                self.expected_nullifiers,
                self.expected_note_logs,
                self.transient_nullifier_indexes_for_note_hashes,
                self.transient_note_hash_indexes_for_nullifiers,
                self.transient_or_propagated_note_hash_indexes_for_logs
            );
        }
    }

    #[test]
    fn succeeds_clear_all() {
        TestDataBuilder::new_clear_all().verify();
    }

    #[test]
    fn succeeds_with_propagated_values() {
        TestDataBuilder::new().verify();
    }

    #[test]
    fn succeeds_partially_propagated() {
        let mut builder = TestDataBuilder::new_clear_all();

        // Keep the note at index 1.
        builder.transient_nullifier_indexes_for_note_hashes[1] = builder.nullifiers.len();
        builder.expected_note_hashes[0] = builder.note_hashes[1];

        // Keep the nullifier at index 2.
        builder.transient_note_hash_indexes_for_nullifiers[2] = builder.note_hashes.len();
        builder.expected_nullifiers[0] = builder.nullifiers[2];

        // Keep the log at index 2.
        builder.transient_or_propagated_note_hash_indexes_for_logs[2] = 0; // Point it to the expected not hash at index 0.
        builder.expected_note_logs[0] = builder.note_logs[2];

        builder.verify();
    }

    #[test]
    fn succeeds_identical_note_hashes() {
        TestDataBuilder::new_identical_note_hashes().verify();
    }

    #[test(should_fail_with="Value of the hinted transient note hash does not match")]
    fn fails_mismatch_note_hash_value() {
        let mut builder = TestDataBuilder::new_clear_all();

        builder.note_hashes[1].note_hash.value += 1;

        builder.verify();
    }

    #[test(should_fail_with="Contract address of the hinted transient note hash does not match")]
    fn fails_mismatch_contract_address() {
        let mut builder = TestDataBuilder::new_clear_all();

        builder.note_hashes[1].contract_address.inner += 1;

        builder.verify();
    }

    #[test(should_fail_with="Empty note hash must be padded to the right")]
    fn fails_unexpected_note_hash_value() {
        let mut builder = TestDataBuilder::new_clear_all();

        builder.expected_note_hashes[2].note_hash.value = 11;

        builder.verify();
    }

    #[test(should_fail_with="Propagated note hash does not match")]
    fn fails_wrong_expected_note_hash_value() {
        let mut builder = TestDataBuilder::new();

        builder.expected_note_hashes[0].note_hash.value += 1;

        builder.verify();
    }

    #[test(should_fail_with="Propagated note hash does not match")]
    fn fails_wrong_expected_note_hash_counter() {
        let mut builder = TestDataBuilder::new();

        builder.expected_note_hashes[0].note_hash.counter += 1;

        builder.verify();
    }

    #[test(should_fail_with="Invalid transient nullifier index hint")]
    fn fails_wrong_hint_for_transient_nullifier_index() {
        let mut builder = TestDataBuilder::new_clear_all();

        builder.transient_note_hash_indexes_for_nullifiers[0] = 1;

        builder.verify();
    }

    #[test(should_fail_with="Empty nullifier must be padded to the right")]
    fn fails_unexpected_nullifier_value() {
        let mut builder = TestDataBuilder::new_clear_all();

        builder.expected_nullifiers[2].nullifier.value = 11;

        builder.verify();
    }

    #[test(should_fail_with="Propagated nullifier does not match")]
    fn fails_wrong_expected_nullifier_value() {
        let mut builder = TestDataBuilder::new();

        builder.expected_nullifiers[0].nullifier.value += 1;

        builder.verify();
    }

    #[test(should_fail_with="Propagated nullifier does not match")]
    fn fails_wrong_expected_nullifier_counter() {
        let mut builder = TestDataBuilder::new();

        builder.expected_nullifiers[0].nullifier.counter += 1;

        builder.verify();
    }

    #[test(should_fail_with="Empty log must be padded to the right")]
    fn fails_unexpected_log_value() {
        let mut builder = TestDataBuilder::new_clear_all();

        builder.expected_note_logs[2].value = 1;

        builder.verify();
    }

    #[test(should_fail_with="Propagated note log does not match")]
    fn fails_wrong_expected_log_value() {
        let mut builder = TestDataBuilder::new();

        builder.expected_note_logs[0].value += 1;

        builder.verify();
    }

    #[test(should_fail_with="Propagated note log does not match")]
    fn fails_wrong_expected_log_counter() {
        let mut builder = TestDataBuilder::new();

        builder.expected_note_logs[0].counter += 1;

        builder.verify();
    }

    #[test(should_fail_with="Hinted transient note log not nullified")]
    fn fails_log_not_nullified() {
        let mut builder = TestDataBuilder::new();

        builder.transient_or_propagated_note_hash_indexes_for_logs[1] = 1;

        builder.verify();
    }

    #[test(should_fail_with="Value of the hinted transient note hash does not match log")]
    fn fails_wrong_log_note_hash() {
        let mut builder = TestDataBuilder::new();

        builder.note_logs[0].note_hash_counter += 1;

        builder.verify();
    }

    #[test(should_fail_with="Empty log must be padded to the right")]
    fn fails_propagate_too_many_logs() {
        let mut builder = TestDataBuilder::new_clear_all();

        // Keep the log.
        builder.expected_note_logs[0] = builder.note_logs[1];

        builder.verify();
    }

    #[test(should_fail_with="Value of the hinted transient note hash does not match log")]
    fn fails_propagate_log_whose_note_hash_has_been_nullified() {
        let mut builder = TestDataBuilder::new_clear_all();

        // Keep the log.
        builder.expected_note_logs[0] = builder.note_logs[1];
        // Point the log to index 0, which is empty in expected_note_hashes, and does not match the transient_note_hash.
        builder.transient_or_propagated_note_hash_indexes_for_logs[1] = 0;

        builder.verify();
    }

    #[test(should_fail_with="Invalid transient nullifier index hint")]
    fn fails_propagate_more_note_hashes_than_nullifiers() {
        let mut builder = TestDataBuilder::new_clear_all();

        // Keep the note at index 1.
        builder.transient_nullifier_indexes_for_note_hashes[1] = builder.nullifiers.len();
        builder.expected_note_hashes[0] = builder.note_hashes[1];

        builder.verify();
    }

    #[test(should_fail_with="Inconsistent number of note hashes and nullifiers removed")]
    fn fails_propagate_more_nullifiers_than_note_hashes() {
        let mut builder = TestDataBuilder::new_clear_all();

        // Keep the nullifier at index 2.
        builder.transient_note_hash_indexes_for_nullifiers[2] = builder.note_hashes.len();
        builder.expected_nullifiers[0] = builder.nullifiers[2];

        builder.verify();
    }

    #[test(should_fail_with="Invalid transient nullifier index hint")]
    fn fails_identical_note_hashes_nullify_same_note_hash() {
        let mut builder = TestDataBuilder::new_identical_note_hashes();

        // Make the nullifier at index 2 to also nullify the note hash at index 0.
        builder.transient_note_hash_indexes_for_nullifiers[2] = 0;

        builder.verify();
    }
}
