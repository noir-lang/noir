use dep::aztec::{
    context::PrivateContext, protocol_types::{address::AztecAddress},
    note::note_getter_options::NoteGetterOptions, state_vars::PrivateSet,
    encrypted_logs::encrypted_note_emission::encode_and_encrypt_note
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
        // Creates new note for the owner.
        let mut addend_note = ValueNote::new(addend as Field, owner_npk_m_hash);

        // Insert the new note to the owner's set of notes.
        // docs:start:insert
        self.set.insert(&mut addend_note).emit(encode_and_encrypt_note(self.context, outgoing_viewer, owner));
        // docs:end:insert
    }

    // Very similar to `value_note::utils::decrement`.
    pub fn sub(self, subtrahend: u64, owner: AztecAddress, outgoing_viewer: AztecAddress) {
        let header = self.context.get_header();
        let owner_npk_m_hash = header.get_npk_m_hash(self.context, owner);

        // docs:start:get_notes
        let options = NoteGetterOptions::with_filter(filter_notes_min_sum, subtrahend as Field);
        let notes = self.set.get_notes(options);
        // docs:end:get_notes

        let mut minuend: u64 = 0;
        for i in 0..options.limit {
            if i < notes.len() {
                let note = notes.get_unchecked(i);

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
        self.set.insert(&mut result_note).emit(encode_and_encrypt_note(self.context, outgoing_viewer, owner));
    }
}
