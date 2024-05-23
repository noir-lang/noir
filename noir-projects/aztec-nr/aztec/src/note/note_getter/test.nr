use dep::protocol_types::constants::MAX_NOTE_HASH_READ_REQUESTS_PER_CALL;
use crate::{
    context::PrivateContext,
    note::{
    note_header::NoteHeader,
    note_getter_options::{NoteGetterOptions, Sort, SortOrder, Comparator, PropertySelector},
    note_getter::constrain_get_notes_internal
}
};
use dep::protocol_types::address::AztecAddress;

use crate::test::{helpers::context_builder::ContextBuilder, mocks::mock_note::MockNote};

global contract_address = AztecAddress::from_field(69);
global storage_slot: Field = 42;

fn setup() -> PrivateContext {
    ContextBuilder::new().contract_address(contract_address).private()
}

fn build_valid_note(value: Field) -> MockNote {
    MockNote::new(value).contract_address(contract_address).storage_slot(storage_slot).build()
}

#[test]
    fn validates_note_and_returns_it() {
    let mut context = setup();

    let value = 1337;
    let test_note = build_valid_note(value);

    let mut opt_notes = [Option::none(); MAX_NOTE_HASH_READ_REQUESTS_PER_CALL];
    opt_notes[0] = Option::some(test_note);

    let options = NoteGetterOptions::new();
    let returned = constrain_get_notes_internal(&mut context, storage_slot, opt_notes, options);

    assert_eq(returned[0].unwrap().value, value);
}

#[test(should_fail)]
    fn cannot_return_zero_notes() {
    let mut context = setup();

    let mut opt_notes: [Option<MockNote>; MAX_NOTE_HASH_READ_REQUESTS_PER_CALL] = [Option::none(); MAX_NOTE_HASH_READ_REQUESTS_PER_CALL];

    let mut options = NoteGetterOptions::new();
    let _ = constrain_get_notes_internal(&mut context, storage_slot, opt_notes, options);
}

#[test(should_fail)]
    fn mismatched_address() {
    let mut context = setup();

    let note = MockNote::new(1).storage_slot(storage_slot).build(); // We're not setting the right contract address

    let mut opt_notes = [Option::none(); MAX_NOTE_HASH_READ_REQUESTS_PER_CALL];
    opt_notes[0] = Option::some(note);

    let mut options = NoteGetterOptions::new();
    let _ = constrain_get_notes_internal(&mut context, storage_slot, opt_notes, options);
}

#[test(should_fail)]
    fn mismatched_storage_slot() {
    let mut context = setup();

    let note = MockNote::new(1).storage_slot(storage_slot + 1).build(); // We're not setting the right storage slot

    let mut opt_notes = [Option::none(); MAX_NOTE_HASH_READ_REQUESTS_PER_CALL];
    opt_notes[0] = Option::some(note);

    let mut options = NoteGetterOptions::new();
    let _ = constrain_get_notes_internal(&mut context, storage_slot, opt_notes, options);
}

#[test(should_fail)]
    fn mismatched_selector() {
    let mut context = setup();

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

#[test(should_fail)]
    fn mismatched_desc_sort_order() {
    let mut context = setup();

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

#[test(should_fail)]
    fn mismatched_asc_sort_order() {
    let mut context = setup();

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

#[test]
    fn sparse_notes_array() {
    let mut context = setup();

    let mut opt_notes = [Option::none(); MAX_NOTE_HASH_READ_REQUESTS_PER_CALL];
    opt_notes[1] = Option::some(build_valid_note(0));
    opt_notes[2] = Option::some(build_valid_note(1));
    opt_notes[3] = Option::some(build_valid_note(2));
    opt_notes[5] = Option::some(build_valid_note(3));
    opt_notes[8] = Option::some(build_valid_note(4));
    opt_notes[13] = Option::some(build_valid_note(5));
    opt_notes[21] = Option::some(build_valid_note(6));

    let options = NoteGetterOptions::new();
    let returned = constrain_get_notes_internal(&mut context, storage_slot, opt_notes, options);

    for i in 0..7 {
        assert(returned[i].unwrap().value == i as Field);
    }
}
