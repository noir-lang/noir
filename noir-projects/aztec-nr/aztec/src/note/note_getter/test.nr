use dep::protocol_types::constants::MAX_NOTE_HASH_READ_REQUESTS_PER_CALL;
use crate::{
    context::PrivateContext,
    note::{
    note_header::NoteHeader,
    note_getter_options::{NoteGetterOptions, Sort, SortOrder, Comparator, PropertySelector},
    note_getter::constrain_get_notes_internal
},
    oracle::execution::get_contract_address
};
use dep::protocol_types::address::AztecAddress;

use crate::test::{helpers::test_environment::TestEnvironment, mocks::mock_note::MockNote};

global storage_slot: Field = 42;

fn setup() -> TestEnvironment {
    TestEnvironment::new()
}

fn build_valid_note(value: Field) -> MockNote {
    MockNote::new(value).contract_address(get_contract_address()).storage_slot(storage_slot).build()
}

fn assert_equivalent_vec_and_array<T, let N: u32>(
    vec: BoundedVec<T, N>,
    arr: [Option<T>; N]
) where T: Eq {
    let mut count = 0;

    for i in 0..N {
        if arr[i].is_some() {
            assert_eq(arr[i].unwrap(), vec.get(count));
            count += 1;
        }
    }

    assert_eq(count, vec.len());
}

#[test]
fn processes_single_note() {
    let mut env = setup();
    let mut context = env.private();

    let mut notes_to_constrain = [Option::none(); MAX_NOTE_HASH_READ_REQUESTS_PER_CALL];
    notes_to_constrain[0] = Option::some(build_valid_note(13));

    let options = NoteGetterOptions::new();
    let returned = constrain_get_notes_internal(&mut context, storage_slot, notes_to_constrain, options);

    assert_equivalent_vec_and_array(returned, notes_to_constrain);
    assert_eq(context.note_hash_read_requests.len(), 1);
}

#[test]
fn processes_many_notes() {
    let mut env = setup();
    let mut context = env.private();

    let mut notes_to_constrain = [Option::none(); MAX_NOTE_HASH_READ_REQUESTS_PER_CALL];
    notes_to_constrain[0] = Option::some(build_valid_note(13));
    notes_to_constrain[1] = Option::some(build_valid_note(19));

    let options = NoteGetterOptions::new();
    let returned = constrain_get_notes_internal(&mut context, storage_slot, notes_to_constrain, options);

    assert_equivalent_vec_and_array(returned, notes_to_constrain);
    assert_eq(context.note_hash_read_requests.len(), 2);
}

#[test]
fn collapses_notes_at_the_beginning_of_the_array() {
    let mut env = setup();
    let mut context = env.private();

    let mut opt_notes = [Option::none(); MAX_NOTE_HASH_READ_REQUESTS_PER_CALL];
    opt_notes[1] = Option::some(build_valid_note(0));
    opt_notes[2] = Option::some(build_valid_note(1));
    opt_notes[3] = Option::some(build_valid_note(2));
    opt_notes[5] = Option::some(build_valid_note(3));
    opt_notes[8] = Option::some(build_valid_note(4));
    opt_notes[13] = Option::some(build_valid_note(5));

    let options = NoteGetterOptions::new();
    let returned = constrain_get_notes_internal(&mut context, storage_slot, opt_notes, options);

    let mut expected = [Option::none(); MAX_NOTE_HASH_READ_REQUESTS_PER_CALL];
    expected[0] = Option::some(build_valid_note(0));
    expected[1] = Option::some(build_valid_note(1));
    expected[2] = Option::some(build_valid_note(2));
    expected[3] = Option::some(build_valid_note(3));
    expected[4] = Option::some(build_valid_note(4));
    expected[5] = Option::some(build_valid_note(5));

    assert_equivalent_vec_and_array(returned, expected);
}

#[test(should_fail_with="Cannot return zero notes")]
fn rejects_zero_notes() {
    let mut env = setup();
    let mut context = env.private();

    let opt_notes: [Option<MockNote>; MAX_NOTE_HASH_READ_REQUESTS_PER_CALL] = [Option::none(); MAX_NOTE_HASH_READ_REQUESTS_PER_CALL];

    let options = NoteGetterOptions::new();
    let _ = constrain_get_notes_internal(&mut context, storage_slot, opt_notes, options);
}

#[test(should_fail_with="Got more notes than limit.")]
fn rejects_mote_notes_than_limit() {
    let mut env = setup();
    let mut context = env.private();

    let mut opt_notes: [Option<MockNote>; MAX_NOTE_HASH_READ_REQUESTS_PER_CALL] = [Option::none(); MAX_NOTE_HASH_READ_REQUESTS_PER_CALL];
    opt_notes[1] = Option::some(build_valid_note(0));
    opt_notes[2] = Option::some(build_valid_note(1));
    opt_notes[3] = Option::some(build_valid_note(2));

    let mut options = NoteGetterOptions::new();
    options = options.set_limit(2);
    let _ = constrain_get_notes_internal(&mut context, storage_slot, opt_notes, options);
}

#[test]
fn applies_filter_before_constraining() {
    let mut env = setup();
    let mut context = env.private();

    let mut notes_to_constrain = [Option::none(); MAX_NOTE_HASH_READ_REQUESTS_PER_CALL];
    let invalid_note = MockNote::new(13).build(); // This note does not have the correct address or storage slot
    notes_to_constrain[0] = Option::some(invalid_note);
    notes_to_constrain[1] = Option::some(build_valid_note(42));

    let filter_fn = |opt_notes: [Option<MockNote>; MAX_NOTE_HASH_READ_REQUESTS_PER_CALL], _| {
        let mut selected = [Option::none(); MAX_NOTE_HASH_READ_REQUESTS_PER_CALL];

        for i in 0..opt_notes.len() {
            // This will filter the notes so that only the valid one remains
            if opt_notes[i].is_some() & (opt_notes[i].unwrap_unchecked().value == 42) {
                selected[i] = opt_notes[i];
            }
        }
        selected
    };

    let options = NoteGetterOptions::with_filter(filter_fn, ());
    let returned = constrain_get_notes_internal(&mut context, storage_slot, notes_to_constrain, options);

    // Only the note with value 42 should be returned, and moved to the beginning of the array. The other notes were not
    // constrained, and hence validation did not fail.
    let mut expected = [Option::none(); MAX_NOTE_HASH_READ_REQUESTS_PER_CALL];
    expected[0] = Option::some(build_valid_note(42));

    assert_equivalent_vec_and_array(returned, expected);
    assert_eq(context.note_hash_read_requests.len(), 1);
}

#[test(should_fail_with="Mismatch note header contract address.")]
fn rejects_mismatched_address() {
    let mut env = setup();
    let mut context = env.private();

    let note = MockNote::new(1).storage_slot(storage_slot).build(); // We're not setting the right contract address

    let mut opt_notes = [Option::none(); MAX_NOTE_HASH_READ_REQUESTS_PER_CALL];
    opt_notes[0] = Option::some(note);

    let mut options = NoteGetterOptions::new();
    let _ = constrain_get_notes_internal(&mut context, storage_slot, opt_notes, options);
}

#[test(should_fail_with="Mismatch note header storage slot.")]
fn rejects_mismatched_storage_slot() {
    let mut env = setup();
    let mut context = env.private();

    let note = MockNote::new(1).contract_address(get_contract_address()).build(); // We're not setting the right storage slot

    let mut opt_notes = [Option::none(); MAX_NOTE_HASH_READ_REQUESTS_PER_CALL];
    opt_notes[0] = Option::some(note);

    let mut options = NoteGetterOptions::new();
    let _ = constrain_get_notes_internal(&mut context, storage_slot, opt_notes, options);
}

#[test(should_fail_with="Mismatch return note field.")]
fn rejects_mismatched_selector() {
    let mut env = setup();
    let mut context = env.private();

    let value = 10;
    let note = build_valid_note(value);

    let mut opt_notes = [Option::none(); MAX_NOTE_HASH_READ_REQUESTS_PER_CALL];
    opt_notes[0] = Option::some(note);

    let mut options = NoteGetterOptions::new();
    options = options.select(
            PropertySelector { index: 0, offset: 0, length: 32 },
            value + 1,
            Option::some(Comparator.EQ)
        );

    let _ = constrain_get_notes_internal(&mut context, storage_slot, opt_notes, options);
}

#[test(should_fail_with="Return notes not sorted in descending order.")]
fn rejects_mismatched_desc_sort_order() {
    let mut env = setup();
    let mut context = env.private();

    let mut opt_notes = [Option::none(); MAX_NOTE_HASH_READ_REQUESTS_PER_CALL];
    // Notes in ascending order
    opt_notes[0] = Option::some(build_valid_note(1));
    opt_notes[1] = Option::some(build_valid_note(2));

    let mut options = NoteGetterOptions::new();
    options = options.sort(
            PropertySelector { index: 0, offset: 0, length: 32 },
            SortOrder.DESC
        );
    let _ = constrain_get_notes_internal(&mut context, storage_slot, opt_notes, options);
}

#[test(should_fail_with="Return notes not sorted in ascending order.")]
fn rejects_mismatched_asc_sort_order() {
    let mut env = setup();
    let mut context = env.private();

    let mut opt_notes = [Option::none(); MAX_NOTE_HASH_READ_REQUESTS_PER_CALL];
    // Notes in descending order
    opt_notes[0] = Option::some(build_valid_note(2));
    opt_notes[1] = Option::some(build_valid_note(1));

    let mut options = NoteGetterOptions::new();
    options = options.sort(
            PropertySelector { index: 0, offset: 0, length: 32 },
            SortOrder.ASC
        );
    let _ = constrain_get_notes_internal(&mut context, storage_slot, opt_notes, options);
}
