use crate::note::{
    note_hash::{compute_inner_hash, compute_siloed_hash, compute_unique_hash},
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

fn compute_siloed_note_hash<Note, N>(
    note_interface: NoteInterface<Note, N>,
    note_with_header: Note,
) -> Field {
    let get_header = note_interface.get_header;
    let header = get_header(note_with_header);

    let inner_note_hash = compute_inner_note_hash(note_interface, note_with_header);

    compute_siloed_hash(header.contract_address, inner_note_hash)
}

fn compute_unique_siloed_note_hash<Note, N>(
    note_interface: NoteInterface<Note, N>,
    note_with_header: Note,
) -> Field {
    let get_header = note_interface.get_header;
    let header = get_header(note_with_header);

    let siloed_note_hash = compute_siloed_note_hash(note_interface, note_with_header);

    compute_unique_hash(header.nonce, siloed_note_hash)
}

fn compute_note_hash_for_read_or_nullify<Note, N>(
    note_interface: NoteInterface<Note, N>,
    note_with_header: Note,
) -> Field {
    let get_header = note_interface.get_header;
    let header = get_header(note_with_header);

    if (header.nonce == 0) {
        // when nonce is zero, that means we are reading a pending note (doesn't have a nonce yet),
        // so we just read the inner_note_hash (kernel will silo by contract address)
        compute_inner_note_hash(note_interface, note_with_header)
    } else {
        // When nonce is nonzero, that means we are reading a settled note (from tree) created in a
        // previous TX. So we need the unique_siloed_note_hash which has already been hashed with
        // contract address and then nonce. This hash will match the existing leaf in the private
        // data tree, so the kernel can just perform a membership check directly on this hash/leaf.
        compute_unique_siloed_note_hash(note_interface, note_with_header)
    }

}

fn compute_note_hash_and_nullifier<Note, N, S>(
    note_interface: NoteInterface<Note, N>,
    note_header: NoteHeader,
    preimage: [Field; S],
) -> [Field; 4] {
    let deserialize = note_interface.deserialize;
    let set_header = note_interface.set_header;
    let mut note = deserialize(arr_copy_slice(preimage, [0; N], 0));
    set_header(&mut note, note_header);

    let compute_note_hash = note_interface.compute_note_hash;
    let note_hash = compute_note_hash(note);
    let inner_note_hash = compute_inner_hash(note_header.storage_slot, note_hash);

    let siloed_note_hash = compute_siloed_hash(note_header.contract_address, inner_note_hash);

    let unique_siloed_note_hash = compute_unique_hash(note_header.nonce, siloed_note_hash);

    let compute_nullifier = note_interface.compute_nullifier;
    let inner_nullifier = compute_nullifier(note);

    [inner_note_hash, siloed_note_hash, unique_siloed_note_hash, inner_nullifier]
}
