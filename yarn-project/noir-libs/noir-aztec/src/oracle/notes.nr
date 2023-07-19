use crate::note::{
    note_header::NoteHeader,
    note_interface::NoteInterface,
};
use crate::utils::arr_copy_slice;

#[oracle(notifyCreatedNote)]
fn notify_created_note_oracle<N>(
    _storage_slot: Field,
    _preimage: [Field; N],
) -> Field {}

unconstrained fn notify_created_note<N>(
    storage_slot: Field,
    preimage: [Field; N],
) -> Field {
    notify_created_note_oracle(storage_slot, preimage)
}

#[oracle(notifyNullifiedNote)]
fn notify_nullified_note_oracle<N>(
    _storage_slot: Field,
    _nullifier: Field,
    _preimage: [Field; N],
) -> Field {}

unconstrained fn notify_nullified_note<N>(
    storage_slot: Field,
    nullifier: Field,
    preimage: [Field; N],
) -> Field {
    notify_nullified_note_oracle(storage_slot, nullifier, preimage)
}

#[oracle(getNotes)]
fn get_notes_oracle<N, S>(
    _storage_slot: Field,
    _sort_by: [u8; N],
    _sort_order: [u8; N],
    _limit: u32,
    _offset: u32,
    _return_size: u32,
    _zero_fields: [Field; S],
) -> [Field; S] {}

unconstrained fn get_notes_oracle_wrapper<N, S>(
    storage_slot: Field,
    sort_by: [u8; N],
    sort_order: [u8; N],
    limit: u32,
    offset: u32,
    mut fields: [Field; S],
)-> [Field; S] {
    let return_size = fields.len() as u32;
    get_notes_oracle(storage_slot, sort_by, sort_order, limit, offset, return_size, fields)
}

// TODO: The following functions should all be unconstrained.

fn get_note<Note, N, NS>(
    storage_slot: Field,
    note_interface: NoteInterface<Note, N>,
    zero_fields: [Field; NS],
) -> Note {
    let fields = get_notes_oracle_wrapper(storage_slot, [], [], 1, 0, zero_fields);
    let has_note = fields[0] == 1;
    let contract_address = fields[1];
    if (has_note) {
        let contract_address = fields[1];
        let nonce = fields[2];
        let header = NoteHeader { contract_address, nonce, storage_slot };

        let deserialise = note_interface.deserialise;
        let note = deserialise(arr_copy_slice(fields, [0; N], 3));

        let set_header = note_interface.set_header;
        set_header(note, header)
    } else {
        let dummy = note_interface.dummy;
        dummy()
    }
}

fn get_notes<Note, N, S, NS>(
    storage_slot: Field,
    note_interface: NoteInterface<Note, N>,
    sort_by: [u8; N],
    sort_order: [u8; N],
    limit: u32,
    offset: u32,
    mut notes: [Note; S], // TODO: Remove it and use `limit` to initialise the note array.
    zero_fields: [Field; NS], // TODO: Remove it and use `limit` to initialise the note array.
) -> [Note; S] {
    let fields = get_notes_oracle_wrapper(storage_slot, sort_by, sort_order, limit, offset, zero_fields);
    let num_notes = fields[0] as u32;
    let contract_address = fields[1];
    let deserialise = note_interface.deserialise;
    let set_header = note_interface.set_header;
    for i in 0..notes.len() {
        if i as u32 < num_notes {
            let read_offset: comptime Field = 2 + i * (N + 1);
            let nonce = fields[read_offset];
            let header = NoteHeader { contract_address, nonce, storage_slot };

            let preimage = arr_copy_slice(fields, [0; N], read_offset + 1);
            let mut note = deserialise(preimage);

            notes[i] = set_header(note, header);
        };
    };
    notes
}

unconstrained fn is_nullifier_emitted(nullifier: Field) -> bool {
    // TODO
    nullifier == 0
}