use dep::aztec::note::note_getter::view_notes;
use crate::value_note::{VALUE_NOTE_LEN, ValueNoteMethods};

unconstrained fn get_balance(storage_slot: Field) -> Field {
    get_balance_internal(storage_slot, 10, 0)
}

unconstrained fn get_balance_internal(storage_slot: Field, limit: u32, offset: u32) -> Field {
    let mut balance = 0;

    let opt_notes = view_notes(storage_slot, ValueNoteMethods, limit, offset);
    let len = opt_notes.len();
    for i in 0..len {
        if opt_notes[i].is_some() {
            balance += opt_notes[i].unwrap_unchecked().value;
        }
    }
    if (opt_notes[len - 1].is_some()) {
        balance += get_balance_internal(storage_slot, limit, offset + limit);
    }

    balance
}