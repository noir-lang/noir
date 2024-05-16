use dep::aztec::{
    protocol_types::{address::AztecAddress, grumpkin_point::GrumpkinPoint}, context::Context,
    note::note_getter_options::NoteGetterOptions, state_vars::PrivateSet,
    keys::getters::{get_npk_m_hash, get_ivpk_m}
};
use dep::value_note::{filter::filter_notes_min_sum, value_note::ValueNote};

struct EasyPrivateUint {
    context: Context,
    set: PrivateSet<ValueNote>,
    storage_slot: Field,
}

// Holds a note that can act similarly to an int.
impl EasyPrivateUint {
    pub fn new(context: Context, storage_slot: Field) -> Self {
        assert(storage_slot != 0, "Storage slot 0 not allowed. Storage slots must start from 1.");
        let set = PrivateSet { context, storage_slot };
        EasyPrivateUint { context, set, storage_slot }
    }

    // Very similar to `value_note::utils::increment`.
    pub fn add(self, addend: u64, owner: AztecAddress) {
        assert(self.context.public.is_none(), "EasyPrivateUint::add can be called from private only.");
        let context = self.context.private.unwrap();

        let owner_npk_m_hash = get_npk_m_hash(context, owner);
        let owner_ivpk_m = get_ivpk_m(context, owner);
        // Creates new note for the owner.
        let mut addend_note = ValueNote::new(addend as Field, owner_npk_m_hash);

        // Insert the new note to the owner's set of notes.
        // docs:start:insert
        self.set.insert(&mut addend_note, true, owner_ivpk_m);
        // docs:end:insert
    }

    // Very similar to `value_note::utils::decrement`.
    pub fn sub(self, subtrahend: u64, owner: AztecAddress) {
        assert(self.context.public.is_none(), "EasyPrivateUint::sub can be called from private only.");
        let context = self.context.private.unwrap();

        let owner_npk_m_hash = get_npk_m_hash(context, owner);
        let owner_ivpk_m = get_ivpk_m(context, owner);

        // docs:start:get_notes
        let options = NoteGetterOptions::with_filter(filter_notes_min_sum, subtrahend as Field);
        let maybe_notes = self.set.get_notes(options);
        // docs:end:get_notes

        let mut minuend: u64 = 0;
        for i in 0..maybe_notes.len() {
            if maybe_notes[i].is_some() {
                let note = maybe_notes[i].unwrap_unchecked();

                // Ensure the notes are actually owned by the owner (to prevent user from generating a valid proof while
                // spending someone else's notes).
                // TODO (#6312): This will break with key rotation. Fix this. Will not be able to pass this assert after rotating key
                assert(note.npk_m_hash.eq(owner_npk_m_hash));

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
        self.set.insert(&mut result_note, result_value != 0, owner_ivpk_m);
    }
}
