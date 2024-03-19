use dep::aztec::{
    note::{note_header::NoteHeader, note_interface::NoteInterface}, hash::pedersen_hash,
    context::PrivateContext
};

global FIELD_NOTE_LEN: Field = 1;

// A note which stores a field and is expected to be passed around using the `addNote` function.
// WARNING: This Note is not private as it does not contain randomness and hence it can be easy to perform serialized_note
// attack on it.
#[aztec(note)]
struct FieldNote {
    value: Field,
}

impl NoteInterface<FIELD_NOTE_LEN> for FieldNote {

    fn compute_nullifier(self, _context: &mut PrivateContext) -> Field {
        // This note is expected to be shared between users and for this reason can't be nullified using a secret.
        0
    }

    fn compute_nullifier_without_context(self) -> Field {
        // This note is expected to be shared between users and for this reason can't be nullified using a secret.
        0
    }

    fn broadcast(self, context: &mut PrivateContext, slot: Field) {
        assert(
            false, "FieldNote does not support broadcast. Add it to PXE directly using the `.addNote` function."
        );
    }
}

impl FieldNote {
    pub fn new(value: Field) -> Self {
        FieldNote { value, header: NoteHeader::empty() }
    }
}

