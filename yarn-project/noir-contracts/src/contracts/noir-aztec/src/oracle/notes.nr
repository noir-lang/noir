use crate::abi::MAX_READ_REQUESTS;
use crate::types::point::Point;
use crate::utils::arr_copy_slice;

// TODO: Replace returned number of notes (MAX_READ_REQUESTS) with a larger global value.

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

#[oracle(getNotes2)]
fn get_notes_2_oracle(
    _storage_slot: Field,
) -> [Field; 13] {}

unconstrained fn get_notes_2_oracle_wrapper(_storage_slot: Field)-> [Field; 13] {
    get_notes_2_oracle(_storage_slot)
}

// TODO: The following functions should all be unconstrained.

fn get_note<Note, N>(
    storage_slot: Field,
    deserialise: fn ([Field; N]) -> Note,
    dummy: fn () -> Note,
) -> Note {
    // TODO: get_note_oracle
    let fields = get_notes_2_oracle_wrapper(storage_slot);
    deserialise(arr_copy_slice(fields, [0; N], 1))
}

fn get_max_notes<Note, N>(
    storage_slot: Field,
    deserialise: fn ([Field; N]) -> Note,
    dummy: fn () -> Note,
    sort_by: [u8; N],
    sort_order: [u8; N],
    skip: u32,
) -> [Note; MAX_READ_REQUESTS] {
    // TODO: get_max_notes_oracle
    let fields = get_notes_2_oracle_wrapper(storage_slot);
    let preimage1 = arr_copy_slice(fields, [0; N], 1);
    let note1 = deserialise(preimage1);
    let preimage2 = arr_copy_slice(fields, [0; N], 1 + preimage1.len());
    let note2 = deserialise(preimage2);
    [note1, note2, dummy(), dummy()]
}

fn get_notes<Note, N, S, P>(
    storage_slot: Field,
    deserialise: fn ([Field; N]) -> Note,
    dummy: fn () -> Note,
    sort_by: [u8; N],
    sort_order: [u8; N],
    skip: u32,
    filter: fn ([Note; MAX_READ_REQUESTS], P) -> [Note; S],
    filter_args: P,
) -> [Note; S] {
    let notes = get_max_notes(storage_slot, deserialise, dummy, sort_by, sort_order, skip);
    filter(notes, filter_args)
}

#[oracle(viewNotesPage)]
fn view_notes_oracle(
    _storage_slot: Field,
    _limit: Field,
    _offset: Field,
) -> [Field; 61] {}

unconstrained fn view_notes_oracle_wrapper(_storage_slot: Field, _limit: u32, _offset: u32)-> [Field; 61] {
    view_notes_oracle(_storage_slot, _limit as Field, _offset as Field)
}

fn view_notes<Note, N, S>(
    storage_slot: Field,
    deserialise: fn ([Field; N]) -> Note,
    dummy: fn () -> Note,
    limit: u32,
    offset: u32,
    init_arr: fn (Note) -> [Note; S], // TODO: Remove it.
) -> [Note; S] {
    let mut notes = init_arr(dummy());
    let fields = view_notes_oracle_wrapper(storage_slot, limit, offset);
    for i in 0..notes.len() {
        let read_offset: comptime Field = 1 + i * N;
        let preimage = arr_copy_slice(fields, [0; N], read_offset);
        let note = deserialise(preimage);
        notes[i] = note;
    };
    notes
}

unconstrained fn is_nullifier_emitted(nullifier: Field) -> bool {
    // TODO
    nullifier == 0
}