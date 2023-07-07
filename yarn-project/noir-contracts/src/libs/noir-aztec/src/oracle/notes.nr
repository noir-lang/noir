use crate::utils::arr_copy_slice;

#[oracle(notifyCreatedNote)]
fn notify_created_note_oracle<N>(
    _storage_slot: Field,
    _preimage: [Field; N],
) -> [Field; 1] {}

unconstrained fn notify_created_note<N>(
    storage_slot: Field,
    preimage: [Field; N],
) -> Field {
    notify_created_note_oracle(storage_slot, preimage)[0]
}

#[oracle(notifyNullifiedNote)]
fn notify_nullified_note_oracle<N>(
    _storage_slot: Field,
    _nullifier: Field,
    _preimage: [Field; N],
) -> [Field; 1] {}

unconstrained fn notify_nullified_note<N>(
    storage_slot: Field,
    nullifier: Field,
    preimage: [Field; N],
) -> Field {
    notify_nullified_note_oracle(storage_slot, nullifier, preimage)[0]
}

#[oracle(getNote)]
fn get_note_oracle<S>(_storage_slot: Field) -> [Field; S] {}

#[oracle(getNotes)]
fn get_notes_oracle<N, S>(
    _storage_slot: Field,
    _note_size: u32,
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
    let note_size: comptime u32 = N;
    let return_size = fields.len() as u32;
    get_notes_oracle(storage_slot, note_size, sort_by, sort_order, limit, offset, return_size, fields)
}

// TODO: The following functions should all be unconstrained.

fn get_note<Note, N>(
    storage_slot: Field,
    deserialise: fn ([Field; N]) -> Note,
    dummy: fn () -> Note,
) -> Note {
    let fields = get_note_oracle(storage_slot);
    let has_note = fields[0] == 1;
    if (has_note) {
        deserialise(arr_copy_slice(fields, [0; N], 1))
    } else {
        dummy()
    }
}

fn get_notes<Note, N, S, NS>(
    storage_slot: Field,
    deserialise: fn ([Field; N]) -> Note,
    dummy: fn () -> Note,
    sort_by: [u8; N],
    sort_order: [u8; N],
    limit: u32,
    offset: u32,
    mut notes: [Note; S], // TODO: Remove it and use `limit` to initialise the note array.
    zero_fields: [Field; NS], // TODO: Remove it and use `limit` to initialise the note array.
) -> [Note; S] {
    let fields = get_notes_oracle_wrapper(storage_slot, sort_by, sort_order, limit, offset, zero_fields);
    let num_notes = fields[0] as u32;
    for i in 0..notes.len() {
        if i as u32 < num_notes {
            let read_offset: comptime Field = 1 + i * N;
            let preimage = arr_copy_slice(fields, [0; N], read_offset);
            let note = deserialise(preimage);
            notes[i] = note;
        };
    };
    notes
}

unconstrained fn is_nullifier_emitted(nullifier: Field) -> bool {
    // TODO
    nullifier == 0
}