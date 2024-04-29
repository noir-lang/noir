use dep::types::{abis::{note_hash::NoteHashContext, nullifier::Nullifier}, traits::is_empty};

pub fn verify_squashed_transient_note_hashes_and_nullifiers<NUM_NOTE_HASHES, NUM_NULLIFIERS>(
    note_hashes: [NoteHashContext; NUM_NOTE_HASHES],
    nullifiers: [Nullifier; NUM_NULLIFIERS],
    expected_note_hashes: [NoteHashContext; NUM_NOTE_HASHES],
    expected_nullifiers: [Nullifier; NUM_NULLIFIERS],
    transient_nullifier_indexes_for_note_hashes: [u64; NUM_NOTE_HASHES],
    transient_note_hash_indexes_for_nullifiers: [u64; NUM_NULLIFIERS]
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
            assert_eq(note_hash.value, nullifier.note_hash, "Hinted note hash does not match");
            assert_eq(
                note_hash.nullifier_counter, nullifier.counter, "Hinted nullifier counter does not match"
            );

            // The following check is not necessary as the nullifier_counter is assured to be greater than the counter of 
            // the note hash when propagated from either the initial or inner private kernel circuits.
            // assert(nullifier.counter > note_hash.counter);

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

            // For each note hash removed, an empty item is padded to the right.
            let padded_nullifier = expected_nullifiers[NUM_NULLIFIERS - nullifiers_removed];
            assert(is_empty(padded_nullifier), "Empty nullifier must be padded to the right");
        }
    }

    assert_eq(
        note_hashes_removed, nullifiers_removed, "Inconsistent number of note hashes and nullifiers removed"
    );
}

mod tests {
    use crate::reset::transient_data::verify_squashed_transient_note_hashes_and_nullifiers;
    use dep::types::{abis::{note_hash::NoteHashContext, nullifier::Nullifier}};

    struct TestDataBuilder<NUM_NOTE_HASHES, NUM_NULLIFIERS> {
        num_note_hashes: u64,
        num_nullifiers: u64,
        note_hashes: [NoteHashContext; NUM_NOTE_HASHES],
        nullifiers: [Nullifier; NUM_NULLIFIERS],
        expected_note_hashes: [NoteHashContext; NUM_NOTE_HASHES],
        expected_nullifiers: [Nullifier; NUM_NULLIFIERS],
        transient_nullifier_indexes_for_note_hashes: [u64; NUM_NOTE_HASHES],
        transient_note_hash_indexes_for_nullifiers: [u64; NUM_NULLIFIERS],
    }

    impl<NUM_NOTE_HASHES, NUM_NULLIFIERS> TestDataBuilder<NUM_NOTE_HASHES, NUM_NULLIFIERS> {
        pub fn default() -> TestDataBuilder<3, 3> {
            let note_hashes = [
                NoteHashContext { value: 11, counter: 100, nullifier_counter: 500 },
                NoteHashContext { value: 22, counter: 200, nullifier_counter: 0 },
                NoteHashContext { value: 33, counter: 300, nullifier_counter: 400 }
            ];

            let nullifiers = [
                Nullifier { value: 44, counter: 400, note_hash: 33 },
                Nullifier { value: 55, counter: 500, note_hash: 11 },
                Nullifier { value: 66, counter: 600, note_hash: 0 }
            ];

            let expected_note_hashes = [note_hashes[1], NoteHashContext::empty(), NoteHashContext::empty()];
            let expected_nullifiers = [nullifiers[2], Nullifier::empty(), Nullifier::empty()];

            let transient_nullifier_indexes_for_note_hashes = [1, 3, 0];
            let transient_note_hash_indexes_for_nullifiers = [2, 0, 3];

            TestDataBuilder {
                num_note_hashes: 3,
                num_nullifiers: 3,
                note_hashes,
                nullifiers,
                expected_note_hashes,
                expected_nullifiers,
                transient_nullifier_indexes_for_note_hashes,
                transient_note_hash_indexes_for_nullifiers
            }
        }

        pub fn default_all_clear() -> TestDataBuilder<3, 3> {
            let note_hashes = [
                NoteHashContext { value: 11, counter: 100, nullifier_counter: 500 },
                NoteHashContext { value: 22, counter: 200, nullifier_counter: 600 },
                NoteHashContext { value: 33, counter: 300, nullifier_counter: 400 }
            ];

            let nullifiers = [
                Nullifier { value: 44, counter: 400, note_hash: 33 },
                Nullifier { value: 55, counter: 500, note_hash: 11 },
                Nullifier { value: 66, counter: 600, note_hash: 22 }
            ];

            let expected_note_hashes = [NoteHashContext::empty(); 3];
            let expected_nullifiers = [Nullifier::empty(); 3];

            let transient_nullifier_indexes_for_note_hashes = [1, 2, 0];
            let transient_note_hash_indexes_for_nullifiers = [2, 0, 1];

            TestDataBuilder {
                num_note_hashes: 3,
                num_nullifiers: 3,
                note_hashes,
                nullifiers,
                expected_note_hashes,
                expected_nullifiers,
                transient_nullifier_indexes_for_note_hashes,
                transient_note_hash_indexes_for_nullifiers
            }
        }

        pub fn verify(self) {
            verify_squashed_transient_note_hashes_and_nullifiers(
                self.note_hashes,
                self.nullifiers,
                self.expected_note_hashes,
                self.expected_nullifiers,
                self.transient_nullifier_indexes_for_note_hashes,
                self.transient_note_hash_indexes_for_nullifiers
            );
        }
    }

    #[test]
    fn verify_squahed_transient_note_hashes_and_nullifiers_with_propagated_values() {
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

        builder.verify();
    }

    #[test(should_fail_with="Hinted note hash does not match")]
    fn mismatch_note_hash_value() {
        let mut builder = TestDataBuilder::default_all_clear();

        builder.note_hashes[1].value += 1;

        builder.verify();
    }

    #[test(should_fail_with="Hinted nullifier counter does not match")]
    fn mismatch_nullifier_counter() {
        let mut builder = TestDataBuilder::default_all_clear();

        builder.note_hashes[1].nullifier_counter += 1;

        builder.verify();
    }

    #[test(should_fail_with="Empty note hash must be padded to the right")]
    fn unexpected_note_hash_value() {
        let mut builder = TestDataBuilder::default_all_clear();

        builder.expected_note_hashes[2].value = 11;

        builder.verify();
    }

    #[test(should_fail_with="Propagated note hash does not match")]
    fn wrong_expected_note_hash_value() {
        let mut builder = TestDataBuilder::default();

        builder.expected_note_hashes[0].value += 1;

        builder.verify();
    }

    #[test(should_fail_with="Propagated note hash does not match")]
    fn wrong_expected_note_hash_counter() {
        let mut builder = TestDataBuilder::default();

        builder.expected_note_hashes[0].counter += 1;

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

        builder.expected_nullifiers[2].value = 11;

        builder.verify();
    }

    #[test(should_fail_with="Propagated nullifier does not match")]
    fn wrong_expected_nullifier_value() {
        let mut builder = TestDataBuilder::default();

        builder.expected_nullifiers[0].value += 1;

        builder.verify();
    }

    #[test(should_fail_with="Propagated nullifier does not match")]
    fn wrong_expected_nullifier_counter() {
        let mut builder = TestDataBuilder::default();

        builder.expected_nullifiers[0].counter += 1;

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
}
