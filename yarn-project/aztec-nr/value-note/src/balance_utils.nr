use dep::aztec::note::{
    note_getter::view_notes,
    note_viewer_options::NoteViewerOptions,
};
use dep::aztec::state_vars::set::Set;
use crate::value_note::{VALUE_NOTE_LEN, ValueNote};

unconstrained fn get_balance(set: Set<ValueNote, VALUE_NOTE_LEN>) -> Field {
    get_balance_with_offset(set, 0)
}

unconstrained fn get_balance_with_offset(set: Set<ValueNote, VALUE_NOTE_LEN>, offset: u32) -> Field {
    let mut balance = 0;
    // docs:start:view_notes
    let options = NoteViewerOptions::new().set_offset(offset);
    let opt_notes = set.view_notes(options);
    // docs:end:view_notes
    let len = opt_notes.len();
    for i in 0..len {
        if opt_notes[i].is_some() {
            balance += opt_notes[i].unwrap_unchecked().value;
        }
    }
    if (opt_notes[len - 1].is_some()) {
        balance += get_balance_with_offset(set, offset + opt_notes.len() as u32);
    }

    balance
}