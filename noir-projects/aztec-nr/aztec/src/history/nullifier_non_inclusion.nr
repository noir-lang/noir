use dep::std::merkle::compute_merkle_root;
use dep::protocol_types::{header::Header, utils::field::{full_field_less_than, full_field_greater_than}};
use crate::{
    context::PrivateContext, note::{utils::compute_siloed_nullifier, note_interface::NoteInterface},
    oracle::get_nullifier_membership_witness::get_low_nullifier_membership_witness
};

trait ProveNullifierNonInclusion {
    fn prove_nullifier_non_inclusion(header: Header, nullifier: Field);
}

impl ProveNullifierNonInclusion for Header {
    fn prove_nullifier_non_inclusion(self, nullifier: Field) {
        // 1) Get the membership witness of a low nullifier of the nullifier
        let witness = get_low_nullifier_membership_witness(self.global_variables.block_number as u32, nullifier);

        // 2) Prove that the nullifier is not included in the nullifier tree

        // 2.a) Compute the low nullifier leaf and prove that it is in the nullifier tree
        let low_nullifier_leaf = witness.leaf_preimage.hash();
        assert(
            self.state.partial.nullifier_tree.root
            == compute_merkle_root(low_nullifier_leaf, witness.index, witness.path), "Proving nullifier non-inclusion failed: Could not prove low nullifier inclusion"
        );

        // 2.b) Prove that the low nullifier is smaller than the nullifier
        assert(
            full_field_less_than(witness.leaf_preimage.nullifier, nullifier), "Proving nullifier non-inclusion failed: low_nullifier.value < nullifier.value check failed"
        );

        // 2.c) Prove that the low nullifier is pointing "over" the nullifier to prove that the nullifier is not
        // included in the nullifier tree (or to 0 if the to-be-inserted nullifier is the largest of all)
        assert(
            full_field_greater_than(witness.leaf_preimage.next_nullifier, nullifier)
            | (witness.leaf_preimage.next_index == 0), "Proving nullifier non-inclusion failed: low_nullifier.next_value > nullifier.value check failed"
        );
        // --> Now we have traversed the trees all the way up to archive root and verified that the nullifier
        //     was not yet included in the nullifier tree.
    }
}

trait ProveNoteNotNullified {
    fn prove_note_not_nullified<Note, N, M>(header: Header, note: Note, context: &mut PrivateContext) where Note: NoteInterface<N, M>;
}

impl ProveNoteNotNullified for Header {
    fn prove_note_not_nullified<Note, N, M>(self, note: Note, context: &mut PrivateContext) where Note: NoteInterface<N, M> {
        let nullifier = compute_siloed_nullifier(note, context);

        self.prove_nullifier_non_inclusion(nullifier);
    }
}
