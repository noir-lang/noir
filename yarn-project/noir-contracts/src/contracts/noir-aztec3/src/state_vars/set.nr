use dep::std;
use crate::abi::PrivateContextInputs;
use crate::context::PrivateFunctionContext;
use crate::notes::value_note::Note;
use crate::notes::value_note::NOTE_LEN;
use crate::types::point::Point;
use crate::utils::arr_copy_slice;

// oracles
use crate::oracle::notes::notify_created_note;
use crate::oracle::notes::notify_nullified_note;
use crate::oracle::notes::get_notes_2_internal;

fn get_2_notes(storage_slot: Field) -> (Note, Note) {
    let fields = get_notes_2_internal(storage_slot);
    // oracle returns count (num notes) in index 0, so first note starts at index 1
    let note1 = Note::deserialize(arr_copy_slice(fields, [0; NOTE_LEN], 1));
    let note2 = Note::deserialize(arr_copy_slice(fields, [0; NOTE_LEN],  1 + NOTE_LEN));
    (note1, note2)
}

struct Set {
    storage_slot: Field,
}

impl Set {
    fn new(storage_slot: Field) -> Set {
        Set { storage_slot }
    }

    fn insert(self, mut context: PrivateFunctionContext, note: Note) -> PrivateFunctionContext {
        let storage_slot = self.storage_slot;

        let note_hash = note.compute_note_hash();

        constrain notify_created_note(storage_slot, note.owner, note) == 0;
        context.push_new_note_hash(note_hash)
    }

    // The `2` in the name is the number of notes returned
    fn get_2(self, mut context: PrivateFunctionContext) -> (PrivateFunctionContext, (Note, Note)) {
        let storage_slot = self.storage_slot;

        let notes = get_2_notes(storage_slot);

        let note0_hash = notes.0.compute_note_hash();
        //let note1_hash = notes.1.compute_note_hash();

        //if notes.0.is_real  {
        //    context = context.push_read_request(note0_hash);
        //}
        //if notes.1.is_real {
        //    context = context.push_read_request(note1_hash);
        //}
        // needed for some reason to avoid compile error?
        //constrain updated_context.args.get(0) != 0x0bad;
        //(updated_context, notes)

        // TODO/FIXME(dbanks12): really should be using the above
        // conditionals to only push 'real' notes to context, but
        // the conditionals do not work right yet.
        // need to include 0-2 notes based on is_real!!!!
        // https://github.com/AztecProtocol/aztec-packages/issues/780
        (
            context
                .push_read_request(note0_hash),
                //.push_read_request(note1_hash),
            notes
        )
    }


    fn remove(self, inputs: PrivateContextInputs, mut context: PrivateFunctionContext, note: Note) -> PrivateFunctionContext {
        let contract_address = inputs.call_context.storage_contract_address;
        let storage_slot = self.storage_slot;

        let note_hash = note.compute_note_hash();
        // TODO uncomment this when we can specify hash index
        // let siloed_note_hash = dep::std::hash::pedersen([
        //     crate::NoteStorageSlot,
        //     note_hash,
        //     storage_slot,
        //     contract_address,
        // ])[0];
        // let nullifier = note.compute_nullifier(siloed_note_hash);
        let nullifier = note.compute_nullifier(note_hash);

        constrain notify_nullified_note(storage_slot, nullifier, note) == 0;
        context.push_new_nullifier(nullifier)
    }
}
