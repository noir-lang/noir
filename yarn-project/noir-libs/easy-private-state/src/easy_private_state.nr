use dep::value_note::{
    filter::get_2_notes,
    value_note::{
        ValueNote,
        ValueNoteInterface,
        VALUE_NOTE_LEN,
    },
};

use dep::aztec::{
    context::Context,
    log::emit_encrypted_log,
    note::note_getter_options::NoteGetterOptions,
    oracle::get_public_key::get_public_key,
    state_vars::set::Set,
    types::point::Point,
};

struct EasyPrivateUint {
    set: Set<ValueNote, VALUE_NOTE_LEN>,
    storage_slot: Field,
}

impl EasyPrivateUint {
    fn new(storage_slot: Field) -> Self {
        let set = Set {
            storage_slot,
            note_interface: ValueNoteInterface,
        };
        EasyPrivateUint {
            set,
            storage_slot,
        }
    }

    // Very similar to `send_note`.
    fn add(
        self,
        context: &mut Context,
        addend: u120,
        owner: Field,
    ) {
        // Creates new note for the owner.
        let mut addend_note = ValueNote::new(addend as Field, owner);

        // Insert the new note to the owner's set of notes.
        self.set.insert(context, &mut addend_note);

        // Emit the newly created encrypted note preimages via oracle calls.
        let owner_key = get_public_key(owner);
        emit_encrypted_log(
            context,
            context.inputs.call_context.storage_contract_address,
            self.set.storage_slot,
            owner,
            owner_key,
            addend_note.serialise(),
        );
    }

    // Very similar to `spend_note`.
    fn sub(
        self,
        context: &mut Context,
        subtrahend: u120,
        owner: Field,
    ) {
        let options = NoteGetterOptions::with_filter(get_2_notes, 0);
        let notes = self.set.get_notes(context, options);
        let note1 = notes[0];
        let note2 = notes[1];

        // Ensure the notes are actually owned by the owner (to prevent user from generating a valid proof while
        // nullifying someone else's notes).
        let validate = |note: ValueNote, owner: Field| {
            let condition = (owner == note.owner);
            assert((!note.is_real) | condition);
        };
        
        validate(note1, owner);
        validate(note2, owner);

        let note1_value: u120 = note1.value as u120;
        let note2_value: u120 = note2.value as u120;
        let minuend = note1_value + note2_value;
        assert(minuend >= subtrahend);

        // Removes the 2 notes from the owner's set of notes.
        self.set.remove(context, note1);
        self.set.remove(context, note2);

        // Creates change note for the owner.
        let result = minuend - subtrahend;
        let mut result_note = ValueNote::new(result as Field, owner);
        self.set.insert(context, &mut result_note);

        // Emit the newly created encrypted note preimages via oracle calls.
        let mut encrypted_data = [0; VALUE_NOTE_LEN];
        if result_note.is_dummy() == false {
            encrypted_data = result_note.serialise();
        };

        let owner_key = get_public_key(owner);

        emit_encrypted_log(
            context,
            context.inputs.call_context.storage_contract_address,
            self.set.storage_slot,
            owner,
            owner_key,
            encrypted_data,
        );
    }
}

