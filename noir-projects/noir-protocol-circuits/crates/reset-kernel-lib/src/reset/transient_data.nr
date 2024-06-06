use dep::types::{
    abis::{note_hash::ScopedNoteHash, nullifier::ScopedNullifier, log_hash::NoteLogHash},
    traits::is_empty
};

pub fn verify_squashed_transient_note_hashes_and_nullifiers<NUM_NOTE_HASHES, NUM_NULLIFIERS, NUM_LOGS>(
    note_hashes: [ScopedNoteHash; NUM_NOTE_HASHES],
    nullifiers: [ScopedNullifier; NUM_NULLIFIERS],
    note_logs: [NoteLogHash; NUM_LOGS],
    expected_note_hashes: [ScopedNoteHash; NUM_NOTE_HASHES],
    expected_nullifiers: [ScopedNullifier; NUM_NULLIFIERS],
    expected_note_logs: [NoteLogHash; NUM_LOGS],
    transient_nullifier_indexes_for_note_hashes: [u32; NUM_NOTE_HASHES],
    transient_note_hash_indexes_for_nullifiers: [u32; NUM_NULLIFIERS],
    transient_note_hash_indexes_for_logs: [u32; NUM_LOGS]
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
            assert_eq(
                note_hash.nullifier_counter, nullifier.counter(), "Nullifier counter of the hinted transient note hash does not match"
            );

            // The following check is not necessary as the nullifier_counter is assured to be greater than the counter of 
            // the note hash when propagated from either the initial or inner private kernel circuits.
            // assert(nullifier.counter > note_hash.counter());

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
        let index_hint = transient_note_hash_indexes_for_logs[i];
        if index_hint == NUM_NOTE_HASHES {
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

            // Below check not required - It links logs to nullifiers, but we have already linked logs to note hashes,
            // and note hashes to nullifiers.
            // assert_eq(
            //     log.note_hash, nullifiers[transient_nullifier_index].nullified_note_hash(), "Value of the hinted transient nullifier does not match log"
            // );

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
    use crate::reset::transient_data::verify_squashed_transient_note_hashes_and_nullifiers;
    use dep::types::{
        abis::{note_hash::{NoteHash, ScopedNoteHash}, nullifier::{Nullifier, ScopedNullifier}, log_hash::NoteLogHash},
        address::AztecAddress
    };

    struct TestDataBuilder<NUM_NOTE_HASHES, NUM_NULLIFIERS, NUM_LOGS> {
        num_note_hashes: u32,
        num_nullifiers: u32,
        note_hashes: [ScopedNoteHash; NUM_NOTE_HASHES],
        nullifiers: [ScopedNullifier; NUM_NULLIFIERS],
        note_logs: [NoteLogHash; NUM_LOGS],
        expected_note_hashes: [ScopedNoteHash; NUM_NOTE_HASHES],
        expected_nullifiers: [ScopedNullifier; NUM_NULLIFIERS],
        expected_note_logs: [NoteLogHash; NUM_LOGS],
        transient_nullifier_indexes_for_note_hashes: [u32; NUM_NOTE_HASHES],
        transient_note_hash_indexes_for_nullifiers: [u32; NUM_NULLIFIERS],
        transient_note_hash_indexes_for_logs: [u32; NUM_LOGS]
    }

    impl<NUM_NOTE_HASHES, NUM_NULLIFIERS, NUM_LOGS> TestDataBuilder<NUM_NOTE_HASHES, NUM_NULLIFIERS, NUM_LOGS> {
        pub fn default() -> TestDataBuilder<3, 3, 2> {
            let contract_address = AztecAddress::from_field(987654);

            let note_hashes = [
                NoteHash { value: 11, counter: 100 }.scope(500, contract_address),
                NoteHash { value: 22, counter: 200 }.scope(0, contract_address),
                NoteHash { value: 33, counter: 300 }.scope(400, contract_address)
            ];

            let nullifiers = [
                Nullifier { value: 44, counter: 400, note_hash: 33 }.scope(contract_address),
                Nullifier { value: 55, counter: 500, note_hash: 11 }.scope(contract_address),
                Nullifier { value: 66, counter: 600, note_hash: 0 }.scope(contract_address)
            ];

            let note_logs = [
                NoteLogHash { value: 77, counter: 700, length: 70, note_hash_counter: 100 },
                NoteLogHash { value: 88, counter: 800, length: 80, note_hash_counter: 200 }
            ];

            let expected_note_hashes = [note_hashes[1], ScopedNoteHash::empty(), ScopedNoteHash::empty()];
            let expected_nullifiers = [nullifiers[2], ScopedNullifier::empty(), ScopedNullifier::empty()];
            let expected_note_logs = [note_logs[1], NoteLogHash::empty()];

            let transient_nullifier_indexes_for_note_hashes = [1, 3, 0];
            let transient_note_hash_indexes_for_nullifiers = [2, 0, 3];
            let transient_note_hash_indexes_for_logs = [0, 3];

            TestDataBuilder {
                num_note_hashes: 3,
                num_nullifiers: 3,
                note_hashes,
                nullifiers,
                note_logs,
                expected_note_hashes,
                expected_nullifiers,
                expected_note_logs,
                transient_nullifier_indexes_for_note_hashes,
                transient_note_hash_indexes_for_nullifiers,
                transient_note_hash_indexes_for_logs
            }
        }

        pub fn default_all_clear() -> TestDataBuilder<3, 3, 4> {
            let contract_address = AztecAddress::from_field(987654);

            let note_hashes = [
                NoteHash { value: 11, counter: 100 }.scope(500, contract_address),
                NoteHash { value: 22, counter: 200 }.scope(600, contract_address),
                NoteHash { value: 33, counter: 300 }.scope(400, contract_address)
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
            let transient_note_hash_indexes_for_logs = [0, 2, 1, 2];

            TestDataBuilder {
                num_note_hashes: 3,
                num_nullifiers: 3,
                note_hashes,
                nullifiers,
                note_logs,
                expected_note_hashes,
                expected_nullifiers,
                expected_note_logs,
                transient_nullifier_indexes_for_note_hashes,
                transient_note_hash_indexes_for_nullifiers,
                transient_note_hash_indexes_for_logs
            }
        }

        pub fn verify(self) {
            let _void = verify_squashed_transient_note_hashes_and_nullifiers(
                self.note_hashes,
                self.nullifiers,
                self.note_logs,
                self.expected_note_hashes,
                self.expected_nullifiers,
                self.expected_note_logs,
                self.transient_nullifier_indexes_for_note_hashes,
                self.transient_note_hash_indexes_for_nullifiers,
                self.transient_note_hash_indexes_for_logs
            );
        }
    }

    #[test]
    fn verify_squashed_transient_note_hashes_and_nullifiers_with_propagated_values() {
        TestDataBuilder::default().verify();
    }

    #[test]
    fn verify_full_transient_note_hashes_and_nullifiers() {
        TestDataBuilder::default_all_clear().verify();
    }

    #[test]
    fn verify_partial_transient_note_hashes_and_nullifiers() {
        let mut builder = TestDataBuilder::default_all_clear();

        // Keep the note at index 1.
        builder.transient_nullifier_indexes_for_note_hashes[1] = builder.num_nullifiers;
        builder.expected_note_hashes[0] = builder.note_hashes[1];

        // Keep the nullifier at index 2.
        builder.transient_note_hash_indexes_for_nullifiers[2] = builder.num_note_hashes;
        builder.expected_nullifiers[0] = builder.nullifiers[2];

        // Keep the log at index 2.
        builder.transient_note_hash_indexes_for_logs[2] = builder.num_note_hashes;
        builder.expected_note_logs[0] = builder.note_logs[2];

        builder.verify();
    }

    #[test(should_fail_with="Value of the hinted transient note hash does not match")]
    fn mismatch_note_hash_value() {
        let mut builder = TestDataBuilder::default_all_clear();

        builder.note_hashes[1].note_hash.value += 1;

        builder.verify();
    }

    #[test(should_fail_with="Contract address of the hinted transient note hash does not match")]
    fn mismatch_contract_address() {
        let mut builder = TestDataBuilder::default_all_clear();

        builder.note_hashes[1].contract_address.inner += 1;

        builder.verify();
    }

    #[test(should_fail_with="Nullifier counter of the hinted transient note hash does not match")]
    fn mismatch_nullifier_counter() {
        let mut builder = TestDataBuilder::default_all_clear();

        builder.note_hashes[1].nullifier_counter += 1;

        builder.verify();
    }

    #[test(should_fail_with="Empty note hash must be padded to the right")]
    fn unexpected_note_hash_value() {
        let mut builder = TestDataBuilder::default_all_clear();

        builder.expected_note_hashes[2].note_hash.value = 11;

        builder.verify();
    }

    #[test(should_fail_with="Propagated note hash does not match")]
    fn wrong_expected_note_hash_value() {
        let mut builder = TestDataBuilder::default();

        builder.expected_note_hashes[0].note_hash.value += 1;

        builder.verify();
    }

    #[test(should_fail_with="Propagated note hash does not match")]
    fn wrong_expected_note_hash_counter() {
        let mut builder = TestDataBuilder::default();

        builder.expected_note_hashes[0].note_hash.counter += 1;

        builder.verify();
    }

    #[test(should_fail_with="Invalid transient nullifier index hint")]
    fn wrong_hint_for_transient_nullifier_index() {
        let mut builder = TestDataBuilder::default_all_clear();

        builder.transient_note_hash_indexes_for_nullifiers[0] = 1;

        builder.verify();
    }

    #[test(should_fail_with="Empty nullifier must be padded to the right")]
    fn unexpected_nullifier_value() {
        let mut builder = TestDataBuilder::default_all_clear();

        builder.expected_nullifiers[2].nullifier.value = 11;

        builder.verify();
    }

    #[test(should_fail_with="Propagated nullifier does not match")]
    fn wrong_expected_nullifier_value() {
        let mut builder = TestDataBuilder::default();

        builder.expected_nullifiers[0].nullifier.value += 1;

        builder.verify();
    }

    #[test(should_fail_with="Propagated nullifier does not match")]
    fn wrong_expected_nullifier_counter() {
        let mut builder = TestDataBuilder::default();

        builder.expected_nullifiers[0].nullifier.counter += 1;

        builder.verify();
    }

    #[test(should_fail_with="Empty log must be padded to the right")]
    fn unexpected_log_value() {
        let mut builder = TestDataBuilder::default_all_clear();

        builder.expected_note_logs[2].value = 1;

        builder.verify();
    }

    #[test(should_fail_with="Propagated note log does not match")]
    fn wrong_expected_log_value() {
        let mut builder = TestDataBuilder::default();

        builder.expected_note_logs[0].value += 1;

        builder.verify();
    }

    #[test(should_fail_with="Propagated note log does not match")]
    fn wrong_expected_log_counter() {
        let mut builder = TestDataBuilder::default();

        builder.expected_note_logs[0].counter += 1;

        builder.verify();
    }

    #[test(should_fail_with="Hinted transient note log not nullified")]
    fn log_not_nullified() {
        let mut builder = TestDataBuilder::default();

        builder.transient_note_hash_indexes_for_logs[1] = 1;

        builder.verify();
    }

    #[test(should_fail_with="Value of the hinted transient note hash does not match log")]
    fn wrong_log_note_hash() {
        let mut builder = TestDataBuilder::default();

        builder.note_logs[0].note_hash_counter += 1;

        builder.verify();
    }

    #[test(should_fail_with="Invalid transient nullifier index hint")]
    fn propagate_more_note_hashes_than_nullifiers() {
        let mut builder = TestDataBuilder::default_all_clear();

        // Keep the note at index 1.
        builder.transient_nullifier_indexes_for_note_hashes[1] = builder.num_nullifiers;
        builder.expected_note_hashes[0] = builder.note_hashes[1];

        builder.verify();
    }

    #[test(should_fail_with="Inconsistent number of note hashes and nullifiers removed")]
    fn propagate_more_nullifiers_than_note_hashes() {
        let mut builder = TestDataBuilder::default_all_clear();

        // Keep the nullifier at index 2.
        builder.transient_note_hash_indexes_for_nullifiers[2] = builder.num_note_hashes;
        builder.expected_nullifiers[0] = builder.nullifiers[2];

        builder.verify();
    }

    #[test(should_fail_with="Empty log must be padded to the right")]
    fn remove_too_many_logs() {
        let mut builder = TestDataBuilder::default();
        // Remove the log at index 1, but don't remove from expected_logs
        builder.note_logs[1].note_hash_counter = 300;
        builder.transient_note_hash_indexes_for_logs[1] = 2;

        builder.verify();
    }
}
