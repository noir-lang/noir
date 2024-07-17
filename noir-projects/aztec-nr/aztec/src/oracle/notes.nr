use crate::note::{note_header::NoteHeader, note_interface::NoteInterface};

use dep::protocol_types::{address::AztecAddress, utils::arr_copy_slice};

#[oracle(notifyCreatedNote)]
unconstrained fn notify_created_note_oracle<let N: u32>(
    _storage_slot: Field,
    _note_type_id: Field,
    _serialized_note: [Field; N],
    _inner_note_hash: Field,
    _counter: u32
) -> Field {}

unconstrained pub fn notify_created_note<let N: u32>(
    storage_slot: Field,
    note_type_id: Field,
    serialized_note: [Field; N],
    inner_note_hash: Field,
    counter: u32
) -> Field {
    notify_created_note_oracle(
        storage_slot,
        note_type_id,
        serialized_note,
        inner_note_hash,
        counter
    )
}

#[oracle(notifyNullifiedNote)]
unconstrained fn notify_nullified_note_oracle<let N: u32>(_nullifier: Field, _inner_note_hash: Field, _counter: u32) -> Field {}

unconstrained pub fn notify_nullified_note<let N: u32>(
    nullifier: Field,
    inner_note_hash: Field,
    counter: u32
) -> Field {
    notify_nullified_note_oracle(nullifier, inner_note_hash, counter)
}

#[oracle(getNotes)]
unconstrained fn get_notes_oracle<let N: u32, let S: u32>(
    _storage_slot: Field,
    _num_selects: u8,
    _select_by_indexes: [u8; N],
    _select_by_offsets: [u8; N],
    _select_by_lengths: [u8; N],
    _select_values: [Field; N],
    _select_comparators: [u8; N],
    _sort_by_indexes: [u8; N],
    _sort_by_offsets: [u8; N],
    _sort_by_lengths: [u8; N],
    _sort_order: [u8; N],
    _limit: u32,
    _offset: u32,
    _status: u8,
    _return_size: u32,
    _placeholder_fields: [Field; S]
) -> [Field; S] {}

unconstrained fn get_notes_oracle_wrapper<let N: u32, let S: u32>(
    storage_slot: Field,
    num_selects: u8,
    select_by_indexes: [u8; N],
    select_by_offsets: [u8; N],
    select_by_lengths: [u8; N],
    select_values: [Field; N],
    select_comparators: [u8; N],
    sort_by_indexes: [u8; N],
    sort_by_offsets: [u8; N],
    sort_by_lengths: [u8; N],
    sort_order: [u8; N],
    limit: u32,
    offset: u32,
    status: u8,
    mut placeholder_fields: [Field; S]
) -> [Field; S] {
    let return_size = placeholder_fields.len() as u32;
    get_notes_oracle(
        storage_slot,
        num_selects,
        select_by_indexes,
        select_by_offsets,
        select_by_lengths,
        select_values,
        select_comparators,
        sort_by_indexes,
        sort_by_offsets,
        sort_by_lengths,
        sort_order,
        limit,
        offset,
        status,
        return_size,
        placeholder_fields
    )
}

unconstrained pub fn get_notes<Note, let N: u32, let NB: u32, let M: u32, let S: u32, let NS: u32>(
    storage_slot: Field,
    num_selects: u8,
    select_by_indexes: [u8; M],
    select_by_offsets: [u8; M],
    select_by_lengths: [u8; M],
    select_values: [Field; M],
    select_comparators: [u8; M],
    sort_by_indexes: [u8; M],
    sort_by_offsets: [u8; M],
    sort_by_lengths: [u8; M],
    sort_order: [u8; M],
    limit: u32,
    offset: u32,
    status: u8,
    mut placeholder_opt_notes: [Option<Note>; S], // TODO: Remove it and use `limit` to initialize the note array.
    placeholder_fields: [Field; NS], // TODO: Remove it and use `limit` to initialize the note array.
    _placeholder_note_length: [Field; N] // Turbofish hack? Compiler breaks calculating read_offset unless we add this parameter
) -> [Option<Note>; S] where Note: NoteInterface<N, NB> {
    let fields = get_notes_oracle_wrapper(
        storage_slot,
        num_selects,
        select_by_indexes,
        select_by_offsets,
        select_by_lengths,
        select_values,
        select_comparators,
        sort_by_indexes,
        sort_by_offsets,
        sort_by_lengths,
        sort_order,
        limit,
        offset,
        status,
        placeholder_fields
    );
    let num_notes = fields[0] as u32;
    let contract_address = AztecAddress::from_field(fields[1]);
    for i in 0..placeholder_opt_notes.len() {
        if i < num_notes {
            // lengths named as per typescript.
            let return_header_length: u32 = 2; // num_notes & contract_address.
            let extra_preimage_length: u32 = 2; // nonce & note_hash_counter.
            let read_offset: u32 = return_header_length + i * (N + extra_preimage_length);
            let nonce = fields[read_offset];
            let note_hash_counter = fields[read_offset + 1] as u32;
            let header = NoteHeader { contract_address, nonce, storage_slot, note_hash_counter };
            let serialized_note = arr_copy_slice(fields, [0; N], read_offset + 2);
            let mut note = Note::deserialize_content(serialized_note);
            note.set_header(header);
            placeholder_opt_notes[i] = Option::some(note);
        };
    }
    placeholder_opt_notes
}

// Only ever use this in private!
#[oracle(checkNullifierExists)]
unconstrained fn check_nullifier_exists_oracle(_inner_nullifier: Field) -> Field {}

// Only ever use this in private!
unconstrained pub fn check_nullifier_exists(inner_nullifier: Field) -> bool {
    check_nullifier_exists_oracle(inner_nullifier) == 1
}
