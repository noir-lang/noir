use dep::aztec::note::{
    note_hash,
    note_header::NoteHeader,
    note_interface::NoteInterface,
};
use dep::aztec::utils::arr_copy_slice;

fn compute_siloed_note_hash<Note, N>(
    note_interface: NoteInterface<Note, N>,
    note: Note,
) -> Field {
    let get_header = note_interface.get_header;
    let header = get_header(note);
    // assert(header.contract_address != 0);

    let compute_note_hash = note_interface.compute_note_hash;
    let note_hash = compute_note_hash(note);
    let inner_note_hash = note_hash::compute_inner_note_hash(header.storage_slot, note_hash);
    note_hash::compute_siloed_note_hash(header.contract_address, inner_note_hash)
}

fn compute_note_hash_and_nullifier<Note, N, S>(
    contract_address: Field,
    storage_slot: Field,
    note_interface: NoteInterface<Note, N>,
    preimage: [Field; S],
) -> [Field; 2] {
    let deserialise = note_interface.deserialise;
    let mut note = deserialise(arr_copy_slice(preimage, [0; N], 0));

    let set_header = note_interface.set_header;
    let header = NoteHeader { contract_address, storage_slot };
    note = set_header(note, header);

    let compute_note_hash = note_interface.compute_note_hash;
    let note_hash = compute_note_hash(note);
    let inner_note_hash = note_hash::compute_inner_note_hash(storage_slot, note_hash);
    
    let compute_nullifier = note_interface.compute_nullifier;
    let inner_nullifier = compute_nullifier(note);

    [inner_note_hash, inner_nullifier]
}
