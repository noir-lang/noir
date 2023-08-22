use dep::std::option::Option;
use crate::note::{
    note_header::NoteHeader,
    note_interface::NoteInterface,
};
use crate::utils::arr_copy_slice;

#[oracle(notifyCreatedNote)]
fn notify_created_note_oracle<N>(
    _storage_slot: Field,
    _preimage: [Field; N],
    _inner_note_hash: Field,
) -> Field {}

unconstrained fn notify_created_note<N>(
    storage_slot: Field,
    preimage: [Field; N],
    inner_note_hash: Field,
) -> Field {
    notify_created_note_oracle(storage_slot, preimage, inner_note_hash)
}

#[oracle(notifyNullifiedNote)]
fn notify_nullified_note_oracle<N>(
    _storage_slot: Field,
    _nullifier: Field,
    _preimage: [Field; N],
    _inner_note_hash: Field,
) -> Field {}

unconstrained fn notify_nullified_note<N>(
    storage_slot: Field,
    nullifier: Field,
    preimage: [Field; N],
    inner_note_hash: Field,
) -> Field {
    notify_nullified_note_oracle(storage_slot, nullifier, preimage, inner_note_hash)
}

#[oracle(getNotes)]
fn get_notes_oracle<N, S>(
    _storage_slot: Field,
    _num_selects: u8,
    _select_by: [u8; N],
    _select_values: [Field; N],
    _sort_by: [u8; N],
    _sort_order: [u2; N],
    _limit: u32,
    _offset: u32,
    _return_size: u32,
    _placeholder_fields: [Field; S],
) -> [Field; S] {}

unconstrained fn get_notes_oracle_wrapper<N, S>(
    storage_slot: Field,
    num_selects: u8,
    select_by: [u8; N],
    select_values: [Field; N],
    sort_by: [u8; N],
    sort_order: [u2; N],
    limit: u32,
    offset: u32,
    mut placeholder_fields: [Field; S],
)-> [Field; S] {
    let return_size = placeholder_fields.len() as u32;
    get_notes_oracle(storage_slot, num_selects, select_by, select_values, sort_by, sort_order, limit, offset, return_size, placeholder_fields)
}

unconstrained fn get_notes<Note, N, M, S, NS>(
    storage_slot: Field,
    note_interface: NoteInterface<Note, N>,
    num_selects: u8,
    select_by: [u8; M],
    select_values: [Field; M],
    sort_by: [u8; M],
    sort_order: [u2; M],
    limit: u32,
    offset: u32,
    mut placeholder_opt_notes: [Option<Note>; S], // TODO: Remove it and use `limit` to initialise the note array.
    placeholder_fields: [Field; NS], // TODO: Remove it and use `limit` to initialise the note array.
) -> [Option<Note>; S] {
    let fields = get_notes_oracle_wrapper(storage_slot, num_selects, select_by, select_values, sort_by, sort_order, limit, offset, placeholder_fields);
    let num_notes = fields[0] as u32;
    let contract_address = fields[1];
    let deserialise = note_interface.deserialise;
    let set_header = note_interface.set_header;
    for i in 0..placeholder_opt_notes.len() {
        if i as u32 < num_notes {
            // lengths named as per typescript.
            let return_header_length: Field = 2; // num_notes & contract_address.
            let extra_preimage_length: Field = 2; // nonce & is_some.
            let read_offset: Field = return_header_length + i * (N + extra_preimage_length);
            let nonce = fields[read_offset];
            let header = NoteHeader { contract_address, nonce, storage_slot };
            let is_some = fields[read_offset + 1] as bool;
            if is_some {
                let preimage = arr_copy_slice(fields, [0; N], read_offset + 2);
                let mut note = deserialise(preimage);
                set_header(&mut note, header);
                placeholder_opt_notes[i] = Option::some(note);
            }
        };
    };
    placeholder_opt_notes
}

unconstrained fn is_nullifier_emitted(nullifier: Field) -> bool {
    // TODO
    nullifier == 0
}