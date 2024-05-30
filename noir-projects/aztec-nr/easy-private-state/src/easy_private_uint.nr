use dep::aztec::{
    context::PrivateContext, protocol_types::{address::AztecAddress, grumpkin_point::GrumpkinPoint},
    note::note_getter_options::NoteGetterOptions, state_vars::PrivateSet
};
use dep::value_note::{filter::filter_notes_min_sum, value_note::ValueNote};

struct EasyPrivateUint<Context> {
    context: Context,
    set: PrivateSet<ValueNote, Context>,
    storage_slot: Field,
}

// Holds a note that can act similarly to an int.
impl<Context> EasyPrivateUint<Context> {
    pub fn new(context: Context, storage_slot: Field) -> Self {
        assert(storage_slot != 0, "Storage slot 0 not allowed. Storage slots must start from 1.");
        let set = PrivateSet::new(context, storage_slot);
        EasyPrivateUint { context, set, storage_slot }
    }
}

impl<Context> EasyPrivateUint<&mut PrivateContext> {
    // Very similar to `value_note::utils::increment`.
    pub fn add(self, addend: u64, owner: AztecAddress, outgoing_viewer: AztecAddress) {
        let header = self.context.get_header();
        let owner_npk_m_hash = header.get_npk_m_hash(self.context, owner);
        let outgoing_viewer = header.get_ovpk_m(self.context, outgoing_viewer);
        let owner_ivpk_m = header.get_ivpk_m(self.context, owner);
        // Creates new note for the owner.
        let mut addend_note = ValueNote::new(addend as Field, owner_npk_m_hash);

        // Insert the new note to the owner's set of notes.
        // docs:start:insert
        self.set.insert(&mut addend_note, outgoing_viewer, owner_ivpk_m);
        // docs:end:insert
    }

    // Very similar to `value_note::utils::decrement`.
    pub fn sub(self, subtrahend: u64, owner: AztecAddress, outgoing_viewer: AztecAddress) {
        let header = self.context.get_header();
        let owner_npk_m_hash = header.get_npk_m_hash(self.context, owner);
        let outgoing_viewer_ovpk_m = header.get_ovpk_m(self.context, outgoing_viewer);
        let owner_ivpk_m = header.get_ivpk_m(self.context, owner);

        // docs:start:get_notes
        let options = NoteGetterOptions::with_filter(filter_notes_min_sum, subtrahend as Field);
        let maybe_notes = self.set.get_notes(options);
        // docs:end:get_notes

        let mut minuend: u64 = 0;
        for i in 0..maybe_notes.len() {
            if maybe_notes[i].is_some() {
                let note = maybe_notes[i].unwrap_unchecked();

                // Removes the note from the owner's set of notes.
                // docs:start:remove
                self.set.remove(note);
                // docs:end:remove

                minuend += note.value as u64;
            }
        }

        assert(minuend >= subtrahend);

        // Creates change note for the owner.
        let result_value = minuend - subtrahend;
        let mut result_note = ValueNote::new(result_value as Field, owner_npk_m_hash);
        self.set.insert(&mut result_note, outgoing_viewer_ovpk_m, owner_ivpk_m);
    }
}
