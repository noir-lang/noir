use dep::std::merkle::compute_merkle_root;
use dep::protocol_types::header::Header;

use crate::{
    context::PrivateContext,
    note::{
        utils::compute_note_hash_for_consumption,
        note_interface::NoteInterface,
    },
    oracle::get_membership_witness::get_note_hash_membership_witness,
};

pub fn _note_inclusion<Note, N>(note: Note, header: Header) where Note: NoteInterface<N> {
    // 1) Compute note_hash
    let note_hash = compute_note_hash_for_consumption(note);

    // 2) Get the membership witness of the note in the note hash tree
    let witness = get_note_hash_membership_witness(header.global_variables.block_number as u32, note_hash);

    // 3) Prove that the commitment is in the note hash tree
    assert_eq(
        header.state.partial.note_hash_tree.root, compute_merkle_root(note_hash, witness.index, witness.path), "Proving note inclusion failed"
    );
}

pub fn prove_note_inclusion<Note, N>(note: Note, context: PrivateContext) where Note: NoteInterface<N> {
    _note_inclusion(note, context.historical_header);
}

pub fn prove_note_inclusion_at<Note, N>(
    note: Note,
    block_number: u32, // The block at which we'll prove that the note exists
    context: PrivateContext
) where Note: NoteInterface<N> {
    let header = context.get_header_at(block_number);

    _note_inclusion(note, header);
}
