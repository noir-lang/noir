use dep::std::hash::pedersen;
use dep::aztec::note::{
    note_header::NoteHeader,
    note_interface::NoteInterface,
    utils::compute_note_hash_for_read_or_nullify,
};

global CLAIM_NOTE_LEN: Field = 2;

struct ClaimNote {
    value: Field,
    secret_hash: Field,
    header: NoteHeader,
}

impl ClaimNote {
    fn new(value: Field, secret_hash: Field) -> Self {
        ClaimNote {
            value,
            secret_hash,
            header: NoteHeader::empty(),
        }
    }

    fn serialize(self) -> [Field; CLAIM_NOTE_LEN] {
        [self.value, self.secret_hash]
    }

    fn deserialize(preimage: [Field; CLAIM_NOTE_LEN]) -> Self {
        ClaimNote {
            value: preimage[0],
            secret_hash: preimage[1],
            header: NoteHeader::empty(),
        }
    }

    fn compute_note_hash(self) -> Field {
        // TODO(#1205) Should use a non-zero generator index.
        dep::std::hash::pedersen([
            self.value, 
            self.secret_hash,
        ])[0]
    }

    fn compute_nullifier(self) -> Field {
        let note_hash_for_nullify = compute_note_hash_for_read_or_nullify(ClaimNoteMethods, self);
        // TODO(#1205) Should use a non-zero generator index.
        dep::std::hash::pedersen([
            note_hash_for_nullify,
            self.secret_hash, // Include the secret_hash again so that the public won't know the note has been claimed.
        ])[0]
    }

    fn set_header(&mut self, header: NoteHeader) {
        self.header = header;
    }
}

fn deserialize(preimage: [Field; CLAIM_NOTE_LEN]) -> ClaimNote {
    ClaimNote::deserialize(preimage)
}

fn serialize(note: ClaimNote) -> [Field; CLAIM_NOTE_LEN] {
    note.serialize()
}

fn compute_note_hash(note: ClaimNote) -> Field {
    note.compute_note_hash()
}

fn compute_nullifier(note: ClaimNote) -> Field {
    note.compute_nullifier()
}

fn get_header(note: ClaimNote) -> NoteHeader {
    note.header
}

fn set_header(note: &mut ClaimNote, header: NoteHeader) {
    note.set_header(header)
}

global ClaimNoteMethods = NoteInterface {
    deserialize,
    serialize,
    compute_note_hash,
    compute_nullifier,
    get_header,
    set_header,
};
