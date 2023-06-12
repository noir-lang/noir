use crate::types::point::Point;
use crate::notes::value_note::Note;

global NOTES_PER_PAGE = 10;

#[oracle(notifyCreatedNote)]
fn notify_created_note_oracle(
    _storage_slot: Field,
    _owner: Point,
    _note: Note,
) -> Field {}

unconstrained fn notify_created_note(
    storage_slot: Field,
    owner: Point,
    note: Note,
) -> Field {
    notify_created_note_oracle(storage_slot, owner, note)
}

#[oracle(notifyNullifiedNote)]
fn notify_nullified_note_oracle(
    _storage_slot: Field,
    _nullifier: Field,
    _note: Note,
) -> Field {}

unconstrained fn notify_nullified_note(
    storage_slot: Field,
    nullifier: Field,
    note: Note,
) -> Field {
    notify_nullified_note_oracle(storage_slot, nullifier, note)
}

#[oracle(getNotes2)]
fn get_notes_2_oracle(
    _storage_slot: Field,
) -> [Field; 13] {}

unconstrained fn get_notes_2_internal(
    storage_slot: Field,
) -> [Field; 13] {
    get_notes_2_oracle(storage_slot)
}

#[oracle(viewNotesPage)]
fn view_notes_page_oracle(
    _storage_slot: Field,
    _limit: Field,
    _offset: Field,
) -> [Field; 61] {}

unconstrained fn view_notes_page_internal(
    storage_slot: Field,
    offset: Field,
) -> [Field; 61] {
    view_notes_page_oracle(storage_slot, NOTES_PER_PAGE, offset)
}