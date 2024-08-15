use dep::types::{
    abis::{note_hash::ScopedNoteHash, nullifier::ScopedNullifier, log_hash::NoteLogHash},
    traits::is_empty
};

struct TransientDataIndexHint {
    nullifier_index: u32,
    note_hash_index: u32,
}

impl TransientDataIndexHint {
    pub fn nada(num_nullifiers: u32, num_note_hashes: u32) -> Self {
        TransientDataIndexHint { nullifier_index: num_nullifiers, note_hash_index: num_note_hashes }
    }
}

pub fn verify_squashed_transient_data_with_hint_indexes<
    let NUM_NOTE_HASHES: u32,
    let NUM_NULLIFIERS: u32,
    let NUM_LOGS: u32,
    let NUM_INDEX_HINTS: u32
>(
    note_hashes: [ScopedNoteHash; NUM_NOTE_HASHES],
    nullifiers: [ScopedNullifier; NUM_NULLIFIERS],
    note_logs: [NoteLogHash; NUM_LOGS],
    expected_note_hashes: [ScopedNoteHash; NUM_NOTE_HASHES],
    expected_nullifiers: [ScopedNullifier; NUM_NULLIFIERS],
    expected_note_logs: [NoteLogHash; NUM_LOGS],
    transient_data_index_hints: [TransientDataIndexHint; NUM_INDEX_HINTS],
    transient_or_propagated_note_hash_indexes_for_logs: [u32; NUM_LOGS],
    split_counter: u32,
    squashed_note_hash_hints: [bool; NUM_NOTE_HASHES],
    squashed_nullifier_hints: [bool; NUM_NULLIFIERS]
) {
    let mut num_squashed = 0;
    for i in 0..NUM_INDEX_HINTS {
        let hint = transient_data_index_hints[i];
        if hint.nullifier_index == NUM_NULLIFIERS {
            assert_eq(hint.note_hash_index, NUM_NOTE_HASHES, "Invalid transient data index hint");
        } else {
            let nullifier = nullifiers[hint.nullifier_index];
            let note_hash = note_hashes[hint.note_hash_index];
            assert_eq(
                note_hash.value(), nullifier.nullified_note_hash(), "Value of the hinted transient note hash does not match"
            );
            assert_eq(
                note_hash.contract_address, nullifier.contract_address, "Contract address of the hinted transient note hash does not match"
            );
            assert(
                nullifier.counter() > note_hash.counter(), "Cannot nullify a note hash created afterwards"
            );
            if nullifier.counter() >= split_counter {
                assert(
                    note_hash.counter() >= split_counter, "Cannot squash a non-revertible note hash with a revertible nullifier"
                );
                // Since the nullifier counter must be larger than the note hash counter, it's not possible to squash a revertible note hash with a non-revertible nullifier.
            }
            num_squashed += 1;

            // Padded note hashes.
            let padded_note_hash = expected_note_hashes[NUM_NOTE_HASHES - num_squashed];
            assert(squashed_note_hash_hints[hint.note_hash_index], "Wrong squashed note hash hint");
            assert(is_empty(padded_note_hash), "Empty note hash must be padded to the right");

            // Padded nullifiers.
            let padded_nullifier = expected_nullifiers[NUM_NULLIFIERS - num_squashed];
            assert(squashed_nullifier_hints[hint.nullifier_index], "Wrong squashed nullifier hint");
            assert(is_empty(padded_nullifier), "Empty nullifier must be padded to the right");
        }
    }

    let mut note_hashes_kept = 0;
    for i in 0..NUM_NOTE_HASHES {
        if !squashed_note_hash_hints[i] {
            assert_eq(
                expected_note_hashes[note_hashes_kept], note_hashes[i], "Propagated note hash does not match"
            );
            note_hashes_kept += 1;
        }
    }
    assert_eq(note_hashes_kept + num_squashed, NUM_NOTE_HASHES, "Wrong number of note hashes removed");

    let mut nullifiers_kept = 0;
    for i in 0..NUM_NULLIFIERS {
        if !squashed_nullifier_hints[i] {
            assert_eq(
                expected_nullifiers[nullifiers_kept], nullifiers[i], "Propagated nullifier does not match"
            );
            nullifiers_kept += 1;
        }
    }
    assert_eq(nullifiers_kept + num_squashed, NUM_NULLIFIERS, "Wrong number of nullifiers removed");

    let mut note_logs_kept = 0;
    let mut note_logs_removed = 0;
    for i in 0..NUM_LOGS {
        let log = note_logs[i];
        let hint_index = transient_or_propagated_note_hash_indexes_for_logs[i];
        // - For non-empty logs, their note_hash_counter can't be 0. It must match the counter of a note hash, which is verified in private_call_data_validator.
        //   Therefor, is_propagated will be false if index_hint incorrectly points to an empty expected_note_hash and fails the check inside the else statement.
        // - For empty logs, we expect is_propagated to be true, making them part of the note_logs_kept.
        //   index_hint might point them to the same empty note. This doesn't matter because we increment the value of 
        //   note_logs_kept and ensure that the correct number of empty logs are present in the expected_note_logs.
        let is_propagated = expected_note_hashes[hint_index].counter() == log.note_hash_counter;
        let expected_index = if is_propagated {
            note_logs_kept
        } else {
            NUM_LOGS - note_logs_removed - 1
        };
        let expected_log = expected_note_logs[expected_index];
        if is_propagated {
            assert_eq(expected_log, log, "Propagated note log does not match");
            note_logs_kept += 1;
        } else {
            let hint = transient_data_index_hints[hint_index];
            let transient_note_hash = note_hashes[hint.note_hash_index];
            assert_eq(
                log.note_hash_counter, transient_note_hash.counter(), "Value of the hinted transient note hash does not match log"
            );
            // For each log removed, an empty item is padded to the right.
            assert(is_empty(expected_log), "Empty log must be padded to the right");
            note_logs_removed += 1;
        }
    }
}

unconstrained pub fn get_squashed_note_hash_hints<let NUM_INDEX_HINTS: u32, let NUM_NOTE_HASHES: u32>(transient_data_index_hints: [TransientDataIndexHint; NUM_INDEX_HINTS]) -> [bool; NUM_NOTE_HASHES] {
    let mut hints = [false; NUM_NOTE_HASHES];
    for i in 0..transient_data_index_hints.len() {
        let note_hash_index = transient_data_index_hints[i].note_hash_index;
        if note_hash_index != NUM_NOTE_HASHES {
            hints[note_hash_index] = true;
        }
    }
    hints
}

unconstrained pub fn get_squashed_nullifier_hints<let NUM_INDEX_HINTS: u32, let NUM_NULLIFIERS: u32>(transient_data_index_hints: [TransientDataIndexHint; NUM_INDEX_HINTS]) -> [bool; NUM_NULLIFIERS] {
    let mut hints = [false; NUM_NULLIFIERS];
    for i in 0..transient_data_index_hints.len() {
        let nullifier_index = transient_data_index_hints[i].nullifier_index;
        if nullifier_index != NUM_NULLIFIERS {
            hints[nullifier_index] = true;
        }
    }
    hints
}

pub fn verify_squashed_transient_data<
    let NUM_NOTE_HASHES: u32,
    let NUM_NULLIFIERS: u32,
    let NUM_LOGS: u32,
    let NUM_INDEX_HINTS: u32
>(
    note_hashes: [ScopedNoteHash; NUM_NOTE_HASHES],
    nullifiers: [ScopedNullifier; NUM_NULLIFIERS],
    note_logs: [NoteLogHash; NUM_LOGS],
    expected_note_hashes: [ScopedNoteHash; NUM_NOTE_HASHES],
    expected_nullifiers: [ScopedNullifier; NUM_NULLIFIERS],
    expected_note_logs: [NoteLogHash; NUM_LOGS],
    transient_data_index_hints: [TransientDataIndexHint; NUM_INDEX_HINTS],
    transient_or_propagated_note_hash_indexes_for_logs: [u32; NUM_LOGS],
    split_counter: u32
) {
    let squashed_note_hash_hints = get_squashed_note_hash_hints(transient_data_index_hints);
    let squashed_nullifier_hints = get_squashed_nullifier_hints(transient_data_index_hints);
    verify_squashed_transient_data_with_hint_indexes(
        note_hashes,
        nullifiers,
        note_logs,
        expected_note_hashes,
        expected_nullifiers,
        expected_note_logs,
        transient_data_index_hints,
        transient_or_propagated_note_hash_indexes_for_logs,
        split_counter,
        squashed_note_hash_hints,
        squashed_nullifier_hints
    );
}

mod tests {
    use crate::reset::transient_data::{
        get_squashed_note_hash_hints, get_squashed_nullifier_hints, TransientDataIndexHint,
        verify_squashed_transient_data, verify_squashed_transient_data_with_hint_indexes
    };
    use dep::types::{
        abis::{note_hash::{NoteHash, ScopedNoteHash}, nullifier::{Nullifier, ScopedNullifier}, log_hash::NoteLogHash},
        address::AztecAddress
    };

    global contract_address = AztecAddress::from_field(987654);

    struct TestDataBuilder<let NUM_NOTE_HASHES: u32, let NUM_NULLIFIERS: u32, let NUM_LOGS: u32, let NUM_INDEX_HINTS: u32> {
        note_hashes: [ScopedNoteHash; NUM_NOTE_HASHES],
        nullifiers: [ScopedNullifier; NUM_NULLIFIERS],
        note_logs: [NoteLogHash; NUM_LOGS],
        expected_note_hashes: [ScopedNoteHash; NUM_NOTE_HASHES],
        expected_nullifiers: [ScopedNullifier; NUM_NULLIFIERS],
        expected_note_logs: [NoteLogHash; NUM_LOGS],
        transient_data_index_hints: [TransientDataIndexHint; NUM_INDEX_HINTS],
        transient_or_propagated_note_hash_indexes_for_logs: [u32; NUM_LOGS],
        split_counter: u32,
    }

    impl<let NUM_NOTE_HASHES: u32, let NUM_NULLIFIERS: u32, let NUM_LOGS: u32, let NUM_INDEX_HINTS: u32> TestDataBuilder<NUM_NOTE_HASHES, NUM_NULLIFIERS, NUM_LOGS, NUM_INDEX_HINTS> {
        pub fn new() -> TestDataBuilder<5, 4, 3, 2> {
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

            let transient_data_index_hints = [
                TransientDataIndexHint { nullifier_index: 0, note_hash_index: 2 },
                TransientDataIndexHint { nullifier_index: 1, note_hash_index: 0 }
            ];
            let transient_or_propagated_note_hash_indexes_for_logs = [1, 0, 1];

            TestDataBuilder {
                note_hashes,
                nullifiers,
                note_logs,
                expected_note_hashes,
                expected_nullifiers,
                expected_note_logs,
                transient_data_index_hints,
                transient_or_propagated_note_hash_indexes_for_logs,
                split_counter: 0
            }
        }

        pub fn new_clear_all() -> TestDataBuilder<3, 3, 4, 3> {
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

            let transient_data_index_hints = [
                TransientDataIndexHint { nullifier_index: 0, note_hash_index: 2 },
                TransientDataIndexHint { nullifier_index: 1, note_hash_index: 0 },
                TransientDataIndexHint { nullifier_index: 2, note_hash_index: 1 }
            ];
            let transient_or_propagated_note_hash_indexes_for_logs = [1, 0, 2, 0];

            TestDataBuilder {
                note_hashes,
                nullifiers,
                note_logs,
                expected_note_hashes,
                expected_nullifiers,
                expected_note_logs,
                transient_data_index_hints,
                transient_or_propagated_note_hash_indexes_for_logs,
                split_counter: 0
            }
        }

        pub fn new_identical_note_hashes() -> TestDataBuilder<3, 3, 4, 3> {
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

            let transient_data_index_hints = [
                TransientDataIndexHint { nullifier_index: 0, note_hash_index: 0 },
                TransientDataIndexHint { nullifier_index: 2, note_hash_index: 1 },
                TransientDataIndexHint { nullifier_index: 3, note_hash_index: 3 }
            ];
            let transient_or_propagated_note_hash_indexes_for_logs = [1, 1, 1, 0];

            TestDataBuilder {
                note_hashes,
                nullifiers,
                note_logs,
                expected_note_hashes,
                expected_nullifiers,
                expected_note_logs,
                transient_data_index_hints,
                transient_or_propagated_note_hash_indexes_for_logs,
                split_counter: 0
            }
        }

        pub fn get_nada_index_hint(_self: Self) -> TransientDataIndexHint {
            TransientDataIndexHint { nullifier_index: NUM_NULLIFIERS, note_hash_index: NUM_NOTE_HASHES }
        }

        pub fn get_hint_indexes(self) -> ([bool; NUM_NOTE_HASHES], [bool; NUM_NULLIFIERS]) {
            let squashed_note_hash_hints = get_squashed_note_hash_hints(self.transient_data_index_hints);
            let squashed_nullifier_hints = get_squashed_nullifier_hints(self.transient_data_index_hints);
            (squashed_note_hash_hints, squashed_nullifier_hints)
        }

        pub fn verify(self) {
            verify_squashed_transient_data(
                self.note_hashes,
                self.nullifiers,
                self.note_logs,
                self.expected_note_hashes,
                self.expected_nullifiers,
                self.expected_note_logs,
                self.transient_data_index_hints,
                self.transient_or_propagated_note_hash_indexes_for_logs,
                self.split_counter
            );
        }

        pub fn verify_with_hint_indexes(
            self,
            squashed_note_hash_hints: [bool; NUM_NOTE_HASHES],
            squashed_nullifier_hints: [bool; NUM_NULLIFIERS]
        ) {
            verify_squashed_transient_data_with_hint_indexes(
                self.note_hashes,
                self.nullifiers,
                self.note_logs,
                self.expected_note_hashes,
                self.expected_nullifiers,
                self.expected_note_logs,
                self.transient_data_index_hints,
                self.transient_or_propagated_note_hash_indexes_for_logs,
                self.split_counter,
                squashed_note_hash_hints,
                squashed_nullifier_hints
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

        // Keep the nullifier at index 1 and the note hash at index 0.
        let removed_hint = builder.transient_data_index_hints[0];
        assert_eq(removed_hint.nullifier_index, 0);
        assert_eq(removed_hint.note_hash_index, 2);
        // Update the hint to skip squashing.
        builder.transient_data_index_hints[0] = builder.get_nada_index_hint();
        // Propagate the values.
        builder.expected_note_hashes[0] = builder.note_hashes[removed_hint.note_hash_index];
        builder.expected_nullifiers[0] = builder.nullifiers[removed_hint.nullifier_index];

        // Keep the logs for note hash at index 0.
        builder.transient_or_propagated_note_hash_indexes_for_logs[1] = 0; // Point it to the expected not hash at index 0.
        builder.transient_or_propagated_note_hash_indexes_for_logs[3] = 0; // Point it to the expected not hash at index 0.
        builder.expected_note_logs[0] = builder.note_logs[1];
        builder.expected_note_logs[1] = builder.note_logs[3];

        builder.verify();
    }

    #[test]
    fn succeeds_identical_note_hashes() {
        TestDataBuilder::new_identical_note_hashes().verify();
    }

    #[test(should_fail_with="Invalid transient data index hint")]
    fn fails_non_nada_index_hint() {
        let mut builder = TestDataBuilder::new_clear_all();

        builder.transient_data_index_hints[0] = builder.get_nada_index_hint();
        // Assign a value to the note_hash_index in a nada index hint.
        builder.transient_data_index_hints[0].note_hash_index = 1;

        builder.verify();
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

    #[test(should_fail_with="Cannot nullify a note hash created afterwards")]
    fn fails_nullify_note_hash_emitted_afterwards() {
        let mut builder = TestDataBuilder::new();

        // Make the nullifier at index 1 to have a smaller counter than its note hash.
        let hint = builder.transient_data_index_hints[1];
        let note_hash_counter = builder.note_hashes[hint.note_hash_index].counter();
        builder.nullifiers[hint.nullifier_index].nullifier.counter = note_hash_counter - 1;

        builder.verify();
    }

    #[test(should_fail_with="Cannot squash a non-revertible note hash with a revertible nullifier")]
    fn fails_nullify_non_revertible_note_hash_with_revertible_nullifier() {
        let mut builder = TestDataBuilder::new();

        let hint = builder.transient_data_index_hints[1];
        let note_hash_counter = builder.note_hashes[hint.note_hash_index].counter();
        // Make the note hash non-revertible.
        builder.split_counter = note_hash_counter + 1;

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

    #[test(should_fail_with="Wrong squashed note hash hint")]
    fn fails_wrong_note_hash_squashed_hint() {
        let mut builder = TestDataBuilder::new_clear_all();

        let mut (squashed_note_hash_hints, squashed_nullifier_hints) = builder.get_hint_indexes();
        squashed_note_hash_hints[0] = false;

        builder.verify_with_hint_indexes(squashed_note_hash_hints, squashed_nullifier_hints);
    }

    #[test(should_fail_with="Empty note hash must be padded to the right")]
    fn fails_unexpected_note_hash_value() {
        let mut builder = TestDataBuilder::new_clear_all();

        builder.expected_note_hashes[2].note_hash.value = 11;

        builder.verify();
    }

    #[test(should_fail_with="Wrong squashed note hash hint")]
    fn fails_propagate_note_hash_for_squashed_nullifier() {
        let mut builder = TestDataBuilder::new_clear_all();

        // Propagate the note hash at index 1.
        builder.expected_note_hashes[0] = builder.note_hashes[1];

        let mut (squashed_note_hash_hints, squashed_nullifier_hints) = builder.get_hint_indexes();
        // Set the hint to false so it's assumed the note hash has not been squashed.
        squashed_note_hash_hints[1] = false;

        builder.verify_with_hint_indexes(squashed_note_hash_hints, squashed_nullifier_hints);
    }

    #[test(should_fail_with="Empty note hash must be padded to the right")]
    fn fails_identical_note_hashes_nullify_same_note_hash() {
        let mut builder = TestDataBuilder::new_identical_note_hashes();

        assert_eq(builder.transient_data_index_hints[1].note_hash_index, 1);
        // Make the nullifier at index 2 to also nullify the note hash at index 0.
        builder.transient_data_index_hints[1].note_hash_index = 0;
        // Propagate the note hashes at index 1 and 2.
        builder.expected_note_hashes[0] = builder.note_hashes[1];
        builder.expected_note_hashes[1] = builder.note_hashes[2];

        let mut (squashed_note_hash_hints, squashed_nullifier_hints) = builder.get_hint_indexes();
        // Set the hint to false so it's assumed the note hash has not been squashed.
        squashed_note_hash_hints[1] = false;

        builder.verify_with_hint_indexes(squashed_note_hash_hints, squashed_nullifier_hints);
    }

    #[test(should_fail_with="Wrong number of note hashes removed")]
    fn fails_note_hash_not_propagated() {
        let mut builder = TestDataBuilder::new();

        // Do not propagate any note hashes.
        builder.expected_note_hashes[0] = ScopedNoteHash::empty();

        let mut (squashed_note_hash_hints, squashed_nullifier_hints) = builder.get_hint_indexes();
        assert_eq(squashed_note_hash_hints[1], false);
        // Set the hint to true so it's assumed the note hash has been squashed.
        squashed_note_hash_hints[1] = true;

        builder.verify_with_hint_indexes(squashed_note_hash_hints, squashed_nullifier_hints);
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

    #[test(should_fail_with="Wrong squashed nullifier hint")]
    fn fails_wrong_nullifier_hint_index() {
        let mut builder = TestDataBuilder::new_clear_all();

        // Propagate the nullifier at index 1.
        builder.expected_nullifiers[0] = builder.nullifiers[1];

        let mut (squashed_note_hash_hints, squashed_nullifier_hints) = builder.get_hint_indexes();
        // Set the hint to false so it's assumed the note hash has not been squashed.
        squashed_nullifier_hints[1] = false;

        builder.verify_with_hint_indexes(squashed_note_hash_hints, squashed_nullifier_hints);
    }

    #[test(should_fail_with="Empty nullifier must be padded to the right")]
    fn fails_unexpected_nullifier_value() {
        let mut builder = TestDataBuilder::new_clear_all();

        builder.expected_nullifiers[2].nullifier.value = 11;

        builder.verify();
    }

    #[test(should_fail_with="Wrong number of nullifiers removed")]
    fn fails_propagate_nullifier_for_squashed_note_hash() {
        let mut builder = TestDataBuilder::new();

        // Do not propagate any nullifiers.
        builder.expected_nullifiers[0] = ScopedNullifier::empty();

        let mut (squashed_note_hash_hints, squashed_nullifier_hints) = builder.get_hint_indexes();
        // Set the hint to true so it's assumed the note hash has been squashed.
        squashed_nullifier_hints[2] = true;

        builder.verify_with_hint_indexes(squashed_note_hash_hints, squashed_nullifier_hints);
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

    #[test(should_fail_with="Value of the hinted transient note hash does not match log")]
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
        builder.expected_note_logs[1] = builder.note_logs[0];

        builder.verify();
    }
}
