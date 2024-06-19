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
    let notes = set.view_notes(options.set_offset(offset));
    // docs:end:view_notes
    for i in 0..options.limit {
        if i < notes.len() {
            balance += notes.get_unchecked(i).value;
        }
    }

    if (notes.len() == options.limit) {
        balance += get_balance_with_offset(set, offset + options.limit);
    }

    balance
}
