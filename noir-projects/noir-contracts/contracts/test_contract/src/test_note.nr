use dep::aztec::{
    note::{note_header::NoteHeader, note_interface::NoteInterface}, hash::pedersen_hash,
    context::PrivateContext
};

global TEST_NOTE_LEN: Field = 1;
// TEST_NOTE_LENGTH * 32 + 32(storage_slot as bytes) + 32(note_type_id as bytes)
global TEST_NOTE_BYTES_LENGTH: Field = 1 * 32 + 64;

// A note which stores a field and is expected to be passed around using the `addNote` function.
// WARNING: This Note is not private as it does not contain randomness and hence it can be easy to perform 
// serialized_note attack on it. This note has been developed purely for testing purposes so that it can easily be
// manually added to PXE. Do not use for real applications.
#[aztec(note)]
struct TestNote {
    value: Field,
}

impl NoteInterface<TEST_NOTE_LEN, TEST_NOTE_BYTES_LENGTH> for TestNote {

    fn compute_note_hash_and_nullifier(self, _context: &mut PrivateContext) -> (Field, Field) {
        // This note is expected to be shared between users and for this reason can't be nullified using a secret.
        (0, 0)
    }

    fn compute_note_hash_and_nullifier_without_context(self) -> (Field, Field) {
        // This note is expected to be shared between users and for this reason can't be nullified using a secret.
        (0, 0)
    }
}

impl TestNote {
    pub fn new(value: Field) -> Self {
        TestNote { value, header: NoteHeader::empty() }
    }
}

impl Eq for TestNote {
    fn eq(self, other: Self) -> bool {
        self.value == other.value
    }
}
