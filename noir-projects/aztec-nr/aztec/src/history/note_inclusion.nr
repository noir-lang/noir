use dep::std::merkle::compute_merkle_root;
use dep::protocol_types::header::Header;

use crate::{
    note::{utils::compute_note_hash_for_consumption, note_interface::NoteInterface},
    oracle::get_membership_witness::get_note_hash_membership_witness
};

trait ProveNoteInclusion {
    fn prove_note_inclusion<Note, N, M>(header: Header, note: Note) where Note: NoteInterface<N, M>;
}

impl ProveNoteInclusion for Header {
    fn prove_note_inclusion<Note, N, M>(self, note: Note) where Note: NoteInterface<N, M> {
        // 1) Compute note_hash
        let note_hash = compute_note_hash_for_consumption(note);

        // 2) Get the membership witness of the note in the note hash tree
        let witness = get_note_hash_membership_witness(self.global_variables.block_number as u32, note_hash);

        // 3) Prove that the commitment is in the note hash tree
        assert_eq(
            self.state.partial.note_hash_tree.root, compute_merkle_root(note_hash, witness.index, witness.path), "Proving note inclusion failed"
        );
    }
}
