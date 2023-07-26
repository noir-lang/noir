use dep::aztec::note::note_getter::view_notes;
use crate::value_note::{VALUE_NOTE_LEN, ValueNoteInterface};

unconstrained fn get_balance(storage_slot: Field) -> Field {
    get_balance_internal(storage_slot, 10, 0)
}

// TODO(1.5) We don't support yet more than one page of notes
// noir is having issues with loop handling in unconstrained functions
// rewrite to unconstrained and integrate into noir-aztec when fixed
unconstrained fn get_balance_internal(storage_slot: Field, limit: u32, offset: u32) -> Field {
    let mut balance = 0;

    let notes = view_notes(storage_slot, ValueNoteInterface, limit, offset);
    let len = notes.len();
    for i in 0..len {
        balance += notes[i].value;
    }
    // TODO
    // if (notes[len - 1].is_dummy()) {
    //     balance += get_balance(storage_slot, limit, offset + limit);
    // }

    balance
}