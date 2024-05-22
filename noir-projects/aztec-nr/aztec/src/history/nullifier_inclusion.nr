use dep::std::merkle::compute_merkle_root;
use dep::protocol_types::header::Header;

use crate::{
    context::PrivateContext, oracle::get_nullifier_membership_witness::get_nullifier_membership_witness,
    note::{utils::compute_siloed_nullifier, note_interface::NoteInterface}
};

trait ProveNullifierInclusion {
    fn prove_nullifier_inclusion(header: Header, nullifier: Field);
}

impl ProveNullifierInclusion for Header {
    fn prove_nullifier_inclusion(self, nullifier: Field) {
        // 1) Get the membership witness of the nullifier
        let witness = get_nullifier_membership_witness(self.global_variables.block_number as u32, nullifier);

        // 2) Check that the witness we obtained matches the nullifier
        assert(witness.leaf_preimage.nullifier == nullifier, "Nullifier does not match value in witness");

        // 3) Compute the nullifier tree leaf
        let nullifier_leaf = witness.leaf_preimage.hash();

        // 4) Prove that the nullifier is in the nullifier tree
        assert(
            self.state.partial.nullifier_tree.root
            == compute_merkle_root(nullifier_leaf, witness.index, witness.path), "Proving nullifier inclusion failed"
        );
        // --> Now we have traversed the trees all the way up to archive root and verified that the nullifier
        //     was included in the nullifier tree.
    }
}

trait ProveNoteIsNullified {
    fn prove_note_is_nullified<Note, N, M>(header: Header, note: Note, context: &mut PrivateContext) where Note: NoteInterface<N, M>;
}

impl ProveNoteIsNullified for Header {
    fn prove_note_is_nullified<Note, N, M>(self, note: Note, context: &mut PrivateContext) where Note: NoteInterface<N, M> {
        let nullifier = compute_siloed_nullifier(note, context);

        self.prove_nullifier_inclusion(nullifier);
    }
}
