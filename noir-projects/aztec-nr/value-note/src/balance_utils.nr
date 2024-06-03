use dep::aztec::{
    context::UnconstrainedContext, state_vars::PrivateSet,
    note::{note_getter::view_notes, note_viewer_options::NoteViewerOptions}
};
use crate::value_note::ValueNote;

unconstrained pub fn get_balance(set: PrivateSet<ValueNote, UnconstrainedContext>) -> Field {
    get_balance_with_offset(set, 0)
}

unconstrained pub fn get_balance_with_offset(set: PrivateSet<ValueNote, UnconstrainedContext>, offset: u32) -> Field {
    let mut balance = 0;
    // docs:start:view_notes
    let mut options = NoteViewerOptions::new();
    let opt_notes = set.view_notes(options.set_offset(offset));
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
