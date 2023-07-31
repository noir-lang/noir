use crate::note::{
    note_hash::{compute_inner_hash, compute_unique_hash, compute_siloed_hash},
    note_header::NoteHeader,
    note_interface::NoteInterface,
};
use crate::utils::arr_copy_slice;

fn compute_inner_note_hash<Note, N>(
    note_interface: NoteInterface<Note, N>,
    note: Note,
) -> Field {
    let get_header = note_interface.get_header;
    let header = get_header(note);

    let compute_note_hash = note_interface.compute_note_hash;
    let note_hash = compute_note_hash(note);

    compute_inner_hash(header.storage_slot, note_hash)
}

fn compute_unique_note_hash<Note, N>(
    note_interface: NoteInterface<Note, N>,
    note: Note,
) -> Field {
    let get_header = note_interface.get_header;
    let header = get_header(note);

    let inner_note_hash = compute_inner_note_hash(note_interface, note);

    compute_unique_hash(header.nonce, inner_note_hash)
}

fn compute_siloed_note_hash<Note, N>(
    note_interface: NoteInterface<Note, N>,
    note: Note,
) -> Field {
    let get_header = note_interface.get_header;
    let header = get_header(note);

    let unique_note_hash = compute_unique_note_hash(note_interface, note);

    compute_siloed_hash(header.contract_address, unique_note_hash)
}

fn compute_note_hash_and_nullifier<Note, N, S>(
    note_interface: NoteInterface<Note, N>,
    note_header: NoteHeader,
    preimage: [Field; S],
) -> [Field; 4] {
    let deserialise = note_interface.deserialise;
    let set_header = note_interface.set_header;
    let mut note = deserialise(arr_copy_slice(preimage, [0; N], 0));
    set_header(&mut note, note_header);

    let compute_note_hash = note_interface.compute_note_hash;
    let note_hash = compute_note_hash(note);
    let inner_note_hash = compute_inner_hash(note_header.storage_slot, note_hash);

    let unique_note_hash = compute_unique_hash(note_header.nonce, inner_note_hash);

    let siloed_note_hash = compute_siloed_hash(note_header.contract_address, unique_note_hash);
    
    let compute_nullifier = note_interface.compute_nullifier;
    let inner_nullifier = compute_nullifier(note);

    [inner_note_hash, unique_note_hash, siloed_note_hash, inner_nullifier]
}
