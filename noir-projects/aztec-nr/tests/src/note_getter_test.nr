use dep::protocol_types::constants::MAX_NOTE_HASH_READ_REQUESTS_PER_CALL;
use dep::aztec::context::PrivateContext;
use dep::aztec::note::{
    note_header::NoteHeader,
    note_getter_options::{NoteGetterOptions, Sort, SortOrder, Comparator, PropertySelector},
    note_getter::_get_notes_constrain_get_notes_internal
};
use dep::aztec::protocol_types::address::AztecAddress;
use crate::mock::test_note::TestNote;

#[test]
fn sets_note_manually_and_fetches_it() {
    let mut context: PrivateContext = dep::std::unsafe::zeroed();
    context.inputs.call_context.storage_contract_address = AztecAddress::from_field(69);

    let mut test_note = TestNote::new(1337);
    test_note.header = NoteHeader::new(AztecAddress::from_field(69), 0, 42);

    let mut opt_notes = [Option::none(); MAX_NOTE_HASH_READ_REQUESTS_PER_CALL];
    opt_notes[0] = Option::some(test_note);

    let storage_slot: Field = 42;

    let mut options = NoteGetterOptions::new();
    let returned = _get_notes_constrain_get_notes_internal(&mut context, storage_slot, opt_notes, options);
    assert_eq(returned[0].unwrap().value, 1337);
}

#[test(should_fail)]
fn cannot_return_zero_notes() {
    let mut context: PrivateContext = dep::std::unsafe::zeroed();
    let storage_slot: Field = 0;
    let mut opt_notes: [Option<TestNote>; MAX_NOTE_HASH_READ_REQUESTS_PER_CALL] = [Option::none(); MAX_NOTE_HASH_READ_REQUESTS_PER_CALL];

    let mut options = NoteGetterOptions::new();
    let returned = _get_notes_constrain_get_notes_internal(&mut context, storage_slot, opt_notes, options);
}

#[test(should_fail)]
fn mismatched_address() {
    let mut context: PrivateContext = dep::std::unsafe::zeroed();
    context.inputs.call_context.storage_contract_address = AztecAddress::from_field(1);

    let storage_slot: Field = 0;
    let mut opt_notes: [Option<TestNote>; MAX_NOTE_HASH_READ_REQUESTS_PER_CALL] = [Option::none(); MAX_NOTE_HASH_READ_REQUESTS_PER_CALL];
    opt_notes[0] = Option::some(TestNote::new(1));

    let mut options = NoteGetterOptions::new();
    let returned = _get_notes_constrain_get_notes_internal(&mut context, storage_slot, opt_notes, options);
}

#[test(should_fail)]
fn mismatched_storage_slot() {
    let mut context: PrivateContext = dep::std::unsafe::zeroed();
    context.inputs.call_context.storage_contract_address = AztecAddress::from_field(1);

    let mut test_note = TestNote::new(1);
    test_note.header = NoteHeader::new(AztecAddress::from_field(1), 0, 1);

    let mut opt_notes: [Option<TestNote>; MAX_NOTE_HASH_READ_REQUESTS_PER_CALL] = [Option::none(); MAX_NOTE_HASH_READ_REQUESTS_PER_CALL];
    opt_notes[0] = Option::some(test_note);

    let storage_slot: Field = 0;

    let mut options = NoteGetterOptions::new();
    let returned = _get_notes_constrain_get_notes_internal(&mut context, storage_slot, opt_notes, options);
}

#[test(should_fail)]
fn invalid_selector() {
    let mut context: PrivateContext = dep::std::unsafe::zeroed();
    context.inputs.call_context.storage_contract_address = AztecAddress::from_field(1);

    let mut test_note = TestNote::new(1);
    test_note.header = NoteHeader::new(AztecAddress::from_field(1), 0, 0);

    let mut opt_notes: [Option<TestNote>; MAX_NOTE_HASH_READ_REQUESTS_PER_CALL] = [Option::none(); MAX_NOTE_HASH_READ_REQUESTS_PER_CALL];
    opt_notes[0] = Option::some(test_note);

    let storage_slot: Field = 0;

    let mut options = NoteGetterOptions::new().select(
        PropertySelector { index: 0, offset: 0, length: 32 },
        10,
        Option::some(Comparator.EQ)
    );
    let returned = _get_notes_constrain_get_notes_internal(&mut context, storage_slot, opt_notes, options);
}

#[test(should_fail)]
fn invalid_note_order() {
    let mut context: PrivateContext = dep::std::unsafe::zeroed();

    let mut opt_notes: [Option<TestNote>; MAX_NOTE_HASH_READ_REQUESTS_PER_CALL] = [Option::none(); MAX_NOTE_HASH_READ_REQUESTS_PER_CALL];
    opt_notes[0] = Option::some(TestNote::new(1));
    opt_notes[1] = Option::some(TestNote::new(2));

    let storage_slot: Field = 0;

    let mut options = NoteGetterOptions::new().sort(
        PropertySelector { index: 0, offset: 0, length: 32 },
        SortOrder.DESC
    );
    let returned = _get_notes_constrain_get_notes_internal(&mut context, storage_slot, opt_notes, options);
}

#[test]
fn sparse_notes_array() {
    let mut context: PrivateContext = dep::std::unsafe::zeroed();

    let mut opt_notes: [Option<TestNote>; MAX_NOTE_HASH_READ_REQUESTS_PER_CALL] = [Option::none(); MAX_NOTE_HASH_READ_REQUESTS_PER_CALL];
    opt_notes[1] = Option::some(TestNote::new(0));
    opt_notes[2] = Option::some(TestNote::new(1));
    opt_notes[3] = Option::some(TestNote::new(2));
    opt_notes[5] = Option::some(TestNote::new(3));
    opt_notes[8] = Option::some(TestNote::new(4));
    opt_notes[13] = Option::some(TestNote::new(5));
    opt_notes[21] = Option::some(TestNote::new(6));

    let storage_slot: Field = 0;

    let mut options = NoteGetterOptions::new();
    let returned = _get_notes_constrain_get_notes_internal(&mut context, storage_slot, opt_notes, options);

    for i in 0..7 {
        assert(returned[i].unwrap().value == i as Field);
    }
}
