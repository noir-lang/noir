use dep::std;
use crate::abi::PrivateContextInputs;
use crate::context::PrivateFunctionContext;
use crate::notes::value_note::Note;
use crate::types::point::Point;
use crate::state_vars::note_getter_data::NoteGetterData;
use crate::state_vars::note_getter_data::make_note_getter_data;
use crate::state_vars::note_getter_data::note_getter_data_len;

// oracles
use crate::oracle::notes::notify_created_note;
use crate::oracle::notes::notify_nullified_note;
use crate::oracle::notes::get_notes_2_internal;


fn get_2_notes(storage_slot: Field) -> (NoteGetterData, NoteGetterData) {
    let fields = get_notes_2_internal(storage_slot);
    let data1 = make_note_getter_data(fields, 1);
    let data2 = make_note_getter_data(fields, 1 + note_getter_data_len());
    (data1, data2)
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
    fn get_2(self, inputs: PrivateContextInputs, mut context: PrivateFunctionContext) -> (PrivateFunctionContext, (Note, Note)) {
        let storage_slot = self.storage_slot;

        // let note_getter_data: [NoteGetterData; 2] = ORACLE::get_2_notes(contract_address, storage_slot);
        // let note_getter_data = (
        //     NoteGetterData { note: Note::dummy(), leaf_index: 0, sibling_path: [1, 2, 3, 4, 5], root: 1 },
        //     NoteGetterData { note: Note::dummy(), leaf_index: 1, sibling_path: [1, 2, 3, 4, 5], root: 1 },
        // );
        let note_getter_data = get_2_notes(storage_slot);

        // Forwarded vars added due to https://github.com/noir-lang/noir/issues/1088
        // let process_data = |data, storage_slot_forwarded, contract_address_forwarded| {
        //     let NoteGetterData { note, leaf_index, sibling_path, root } = data;
        //     let _ignored = sibling_path;

        //     let note_hash = note.compute_note_hash();
        //     let siloed_note_hash = dep::std::hash::pedersen([
        //         crate::NoteStorageSlot,
        //         note_hash, 
        //         storage_slot_forwarded, 
        //         contract_address_forwarded           ])[0];

        //     // Check existence in tree:
        //     constrain std::merkle::check_membership(
        //         root,
        //         siloed_note_hash,
        //         leaf_index,
        //         sibling_path,
        //     ) == 1;

        //     // Set the public input of the 'old private data tree root' to be the computed root (if it hasn't already been set); otherwise constrain equality with the previously-set value.
        //     if context.private_data_tree_root == 0 {
        //         context.private_data_tree_root = root;
        //     } else {
        //         constrain context.private_data_tree_root == root;
        //     }
        // };
        // process_data(note_getter_data.0, storage_slot, contract_address);
        // process_data(note_getter_data.1, storage_slot, contract_address);

        constrain note_getter_data.0.root == inputs.roots.private_data_tree_root;
        constrain note_getter_data.1.root == inputs.roots.private_data_tree_root;

        let notes = (note_getter_data.0.note, note_getter_data.1.note);

        (context, notes)
    }


    fn remove(self, inputs: PrivateContextInputs, mut context: PrivateFunctionContext, note: Note) -> PrivateFunctionContext {
        let contract_address = inputs.call_context.storage_contract_address;
        let storage_slot = self.storage_slot;

        let note_hash = note.compute_note_hash();
        let siloed_note_hash = dep::std::hash::pedersen([
            crate::NoteStorageSlot,
            note_hash,
            storage_slot,
            contract_address,
        ])[0];

        let mut nullifier = note.compute_nullifier(siloed_note_hash);
        // TODO remove this when we can specify hash index
        nullifier = note.compute_nullifier(note_hash);

        constrain notify_nullified_note(storage_slot, nullifier, note) == 0;
        context.push_new_nullifier(nullifier)
    }
}
