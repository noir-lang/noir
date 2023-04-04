use dep::std;
use crate::abi::Inputs;
use crate::abi::IntermediateOutputs;
use crate::note::Note;
use crate::note::Point;

#[builtin(notifyCreatedNote)]
fn notify_created_note(
    _storage_slot: Field,
    _owner: Point,
    _note: Note,
) -> Field {}

#[builtin(notifyNullifiedNote)]
fn notify_nullified_note(
    _storage_slot: Field,
    _nullifier: Field,
    _note: Note,
) -> Field {}

#[builtin(getNotes2)]
fn get_notes_2_internal(
    _contract_address: Field,
    _storage_slot: Field,
) -> [Field; 26] {}

// Data returned when getting a note
struct NoteGetterData {
    note: Note,
    leaf_index: Field,
    sibling_path: [Field; crate::TREE_HEIGHT],
    root: Field,
}

fn make_note_getter_data(fields: [Field], start: comptime Field) -> NoteGetterData {
    NoteGetterData {
        note: Note {
            value: fields[start],
            owner: Point::new(fields[start + 1], fields[start + 2]),
            randomness: fields[start + 3],
            nonce: fields[start + 4],
            is_dummy: fields[start + 5] as bool,
        },
        leaf_index: fields[start + 6],
        sibling_path: [
            fields[start + 7],
            fields[start + 8],
            fields[start + 9],
            fields[start + 10],
            fields[start + 11],
        ],
        root: fields[start + 12],
    }
}

fn get_2_notes(contract_address: Field, storage_slot: Field) -> (NoteGetterData, NoteGetterData) {
    let fields = get_notes_2_internal(contract_address, storage_slot);
    let data1 = make_note_getter_data(fields, 0);
    let data2 = make_note_getter_data(fields, 13);
    (data1, data2)
}

struct Set {
    storage_slot: Field,
}

impl Set {
    fn new(storage_slot: Field) -> Set {
        Set { storage_slot }
    }

    fn insert(self, inputs: Inputs, mut outputs: IntermediateOutputs, note: Note) -> IntermediateOutputs {
        let contract_address = inputs.call_context.storage_contract_address;
        let storage_slot = self.storage_slot;

        let note_hash = note.compute_note_hash();
        let siloed_note_hash = dep::std::hash::pedersen([
            crate::NoteStorageSlot,
            note_hash, 
            storage_slot, 
            contract_address,
        ])[0];

        constrain notify_created_note(storage_slot, note.owner, note) == 0;
        outputs = outputs.push_new_note_hash(siloed_note_hash);
        outputs.push_new_note(note)
    }

    // The `2` in the name is the number of notes returned
    fn get_2(self, inputs: Inputs, mut outputs: IntermediateOutputs) -> (IntermediateOutputs, (Note, Note)) {
        let contract_address = inputs.call_context.storage_contract_address;
        let storage_slot = self.storage_slot;

        // let note_getter_data: [NoteGetterData; 2] = ORACLE::get_2_notes(contract_address, storage_slot);
        // let note_getter_data = (
        //     NoteGetterData { note: Note::dummy(), leaf_index: 0, sibling_path: [1, 2, 3, 4, 5], root: 1 },
        //     NoteGetterData { note: Note::dummy(), leaf_index: 1, sibling_path: [1, 2, 3, 4, 5], root: 1 },
        // );
        let note_getter_data = get_2_notes(contract_address, storage_slot);

        // Forwarded vars added due to https://github.com/noir-lang/noir/issues/1088
        let process_data = |data, storage_slot_forwarded, contract_address_forwarded| {
            let NoteGetterData { note, leaf_index, sibling_path, root } = data;
            let _ignored = sibling_path;

            let note_hash = note.compute_note_hash();
            let siloed_note_hash = dep::std::hash::pedersen([
                crate::NoteStorageSlot,
                note_hash, 
                storage_slot_forwarded, 
                contract_address_forwarded           ])[0];

            // Check existence in tree:
            constrain std::merkle::check_membership(
                root,
                siloed_note_hash,
                leaf_index,
                sibling_path,
            ) == 1;

            // Set the public input of the 'old private data tree root' to be the computed root (if it hasn't already been set); otherwise constrain equality with the previously-set value.
            if outputs.old_private_data_tree_root == 0 {
                outputs.old_private_data_tree_root = root;
            } else {
                constrain outputs.old_private_data_tree_root == root;
            }
        };
        process_data(note_getter_data.0, storage_slot, contract_address);
        process_data(note_getter_data.1, storage_slot, contract_address);

        let notes = (note_getter_data.0.note, note_getter_data.1.note);
        (outputs, notes)
    }


    fn remove(self, inputs: Inputs, mut outputs: IntermediateOutputs, note: Note) -> IntermediateOutputs {
        let contract_address = inputs.call_context.storage_contract_address;
        let storage_slot = self.storage_slot;

        let note_hash = note.compute_note_hash();
        let siloed_note_hash = dep::std::hash::pedersen([
            crate::NoteStorageSlot,
            note_hash,
            storage_slot,
            contract_address,
        ])[0];

        let nullifier = note.compute_nullifier(siloed_note_hash);
        constrain notify_nullified_note(storage_slot, nullifier, note) == 0;
        outputs = outputs.push_new_nullifier(nullifier);
        outputs.push_nullified_note(note)
    }
}
