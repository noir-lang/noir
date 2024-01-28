use dep::std::merkle::compute_merkle_root;

use crate::{
    context::PrivateContext,
    note::{
        utils::compute_note_hash_for_read_or_nullify,
        note_header::NoteHeader,
        note_interface::NoteInterface,
    },
    oracle::get_membership_witness::get_note_hash_membership_witness,
};

pub fn prove_note_commitment_inclusion(
    note_commitment: Field,
    block_number: u32, // The block at which we'll prove that the note exists
    context: PrivateContext
) {
    // 1) Get block header from oracle and ensure that the block is included in the archive.
    let block_header = context.get_block_header(block_number);

    // 2) Get the membership witness of the note in the note hash tree
    let witness = get_note_hash_membership_witness(block_number, note_commitment);

    // 3) Prove that the commitment is in the note hash tree
    assert(
        block_header.note_hash_tree_root
        == compute_merkle_root(note_commitment, witness.index, witness.path), "Proving note inclusion failed"
    );
    // --> Now we have traversed the trees all the way up to archive root.
}

pub fn prove_note_inclusion<Note, N>(
    note_interface: NoteInterface<Note, N>,
    note_with_header: Note,
    block_number: u32, // The block at which we'll prove that the note exists
    context: PrivateContext
) {
    let note_commitment = compute_note_hash_for_read_or_nullify(note_interface, note_with_header);

    prove_note_commitment_inclusion(note_commitment, block_number, context);
}
