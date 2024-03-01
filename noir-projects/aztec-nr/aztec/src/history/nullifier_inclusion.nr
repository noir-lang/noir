use dep::std::merkle::compute_merkle_root;
use dep::protocol_types::header::Header;

use crate::{
    context::{PrivateContext, ContextInterface}, oracle::get_nullifier_membership_witness::get_nullifier_membership_witness,
    note::{utils::compute_siloed_nullifier, note_interface::NoteInterface}
};

fn _nullifier_inclusion(nullifier: Field, header: Header) {
    // 1) Get the membership witness of the nullifier
    let witness = get_nullifier_membership_witness(header.global_variables.block_number as u32, nullifier);

    // 2) Check that the witness we obtained matches the nullifier
    assert(witness.leaf_preimage.nullifier == nullifier, "Nullifier does not match value in witness");

    // 3) Compute the nullifier tree leaf
    let nullifier_leaf = witness.leaf_preimage.hash();

    // 4) Prove that the nullifier is in the nullifier tree
    assert(
        header.state.partial.nullifier_tree.root
        == compute_merkle_root(nullifier_leaf, witness.index, witness.path), "Proving nullifier inclusion failed"
    );
    // --> Now we have traversed the trees all the way up to archive root and verified that the nullifier
    //     was included in the nullifier tree.
}

pub fn prove_nullifier_inclusion<TContext>(nullifier: Field, context: TContext) where TContext: ContextInterface {
    _nullifier_inclusion(nullifier, context.get_header());
}

pub fn prove_nullifier_inclusion_at(
    nullifier: Field,
    block_number: u32, // The block at which we'll prove that the nullifier exists in the nullifier tree
    context: PrivateContext
) {
    let header = context.get_header_at(block_number);

    _nullifier_inclusion(nullifier, header);
}

pub fn prove_note_is_nullified<Note, N>(
    note: Note,
    context: &mut PrivateContext
) where Note: NoteInterface<N> {
    let nullifier = compute_siloed_nullifier(note, context);

    _nullifier_inclusion(nullifier, context.historical_header);
}

pub fn prove_note_is_nullified_at<Note, N>(
    note: Note,
    block_number: u32,
    context: &mut PrivateContext
) where Note: NoteInterface<N> {
    let nullifier = compute_siloed_nullifier(note, context);
    let header = context.get_header_at(block_number);

    _nullifier_inclusion(nullifier, header);
}
