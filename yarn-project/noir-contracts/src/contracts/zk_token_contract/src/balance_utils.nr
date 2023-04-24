use dep::aztec3::notes::value_note::Note;
use dep::aztec3::notes::value_note::NOTE_LEN;
use dep::aztec3::types::point::Point;
use dep::aztec3::utils::arr_copy_slice;

global NOTES_PER_PAGE = 10;

#[oracle(viewNotesPage)]
fn view_notes_page_oracle(
    _storage_slot: Field,
    _limit: Field,
    _offset: Field,
) -> [Field; 61] {}

// TODO noir is having issues with array params between unconstrained functions, 
// rewrite to unconstrained and integrate into noir-aztec3 when fixed

unconstrained fn view_notes_page_internal(
    storage_slot: Field,
    offset: Field,
) -> [Field; 61] {
    view_notes_page_oracle(storage_slot, NOTES_PER_PAGE, offset)
}

fn view_notes_page(
    storage_slot: Field,
    offset: Field,
) -> (Field, [Note; NOTES_PER_PAGE]){
    let fields = view_notes_page_internal(storage_slot, offset);
    let total_notes = fields[0];
    let mut notes = [Note::dummy(); NOTES_PER_PAGE];
    for i in 0..NOTES_PER_PAGE {
        let read_offset: comptime Field = 1 + i*NOTE_LEN;
        let note = Note::deserialize(arr_copy_slice(fields, [0; NOTE_LEN], read_offset));
        notes[i] = note;
    };
    (total_notes, notes)
}



fn get_balance(storage_slot: Field) -> Field {
    let mut balance = 0;

    let mut current_batch = view_notes_page(storage_slot, 0).1;
    
    for j in 0..current_batch.len() {
        balance += current_batch[j].value;
    }

    balance
}