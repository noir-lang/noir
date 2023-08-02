use dep::aztec::note::note_getter::view_notes;
use crate::value_note::{VALUE_NOTE_LEN, ValueNoteMethods};

unconstrained fn get_balance(storage_slot: Field) -> Field {
    get_balance_internal(storage_slot, 10, 0)
}

// TODO(1.5) We don't support yet more than one page of notes
// noir is having issues with loop handling in unconstrained functions
// rewrite to unconstrained and integrate into noir-aztec when fixed
unconstrained fn get_balance_internal(storage_slot: Field, limit: u32, offset: u32) -> Field {
    let mut balance = 0;

    let opt_notes = view_notes(storage_slot, ValueNoteMethods, limit, offset);
    let len = opt_notes.len();
    let dummy = ValueNoteMethods.dummy;
    for i in 0..len {
        balance += opt_notes[i].unwrap_or(dummy()).value;
    }
    // TODO
    // if (notes[len - 1].is_dummy()) {
    //     balance += get_balance(storage_slot, limit, offset + limit);
    // }

    balance
}