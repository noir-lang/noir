use dep::aztec::{
    context::{PrivateContext, PublicContext, Context},
    log::emit_encrypted_log,
    note::note_getter_options::NoteGetterOptions,
    oracle::get_public_key::get_public_key,
    state_vars::set::Set,
    types::point::Point,
};
use dep::std::option::Option;
use dep::value_note::{
    filter::filter_notes_min_sum,
    value_note::{ValueNote, ValueNoteMethods, VALUE_NOTE_LEN},
};

struct EasyPrivateUint {
    context: Context,
    set: Set<ValueNote, VALUE_NOTE_LEN>,
    storage_slot: Field,
}

impl EasyPrivateUint {
    fn new(
        context: Context,
        storage_slot: Field,
    ) -> Self {
        assert(storage_slot != 0, "Storage slot 0 not allowed. Storage slots must start from 1.");
        let set = Set {
            context,
            storage_slot,
            note_interface: ValueNoteMethods,
        };
        EasyPrivateUint {
            context,
            set,
            storage_slot,
        }
    }

    // Very similar to `value_note::utils::increment`.
    fn add(self, addend: u120, owner: Field) {
        // Creates new note for the owner.
        let mut addend_note = ValueNote::new(addend as Field, owner);

        // Insert the new note to the owner's set of notes.
        self.set.insert(&mut addend_note);

        // Emit the newly created encrypted note preimages via oracle calls.
        let owner_key = get_public_key(owner);
        let context = self.context.private.unwrap();
        emit_encrypted_log(
            context,
            (*context).this_address(),
            self.set.storage_slot,
            owner_key,
            addend_note.serialise(),
        );
    }

    // Very similar to `value_note::utils::decrement`.
    fn sub(self, subtrahend: u120, owner: Field) {
        let options = NoteGetterOptions::with_filter(filter_notes_min_sum, subtrahend as Field);
        let maybe_notes = self.set.get_notes(options);

        let mut minuend: u120 = 0;
        for i in 0..maybe_notes.len() {
            if maybe_notes[i].is_some() {
                let note = maybe_notes[i].unwrap_unchecked();

                // Ensure the notes are actually owned by the owner (to prevent user from generating a valid proof while
                // spending someone else's notes).
                assert(note.owner == owner);

                // Removes the note from the owner's set of notes.
                self.set.remove(note);

                minuend += note.value as u120;
            }
        }

        assert(minuend >= subtrahend);

        // Creates change note for the owner.
        let result_value = minuend - subtrahend;
        let mut result_note = ValueNote::new(result_value as Field, owner);
        self.set.insert(&mut result_note);

        // Emit the newly created encrypted note preimages via oracle calls.
        let mut encrypted_data = [0; VALUE_NOTE_LEN];
        if result_value != 0 {
            encrypted_data = result_note.serialise();
        };

        let owner_key = get_public_key(owner);

        let context = self.context.private.unwrap();
        emit_encrypted_log(
            context,
            (*context).this_address(),
            self.set.storage_slot,
            owner_key,
            encrypted_data,
        );
    }
}
