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

    fn serialise(self) -> [Field; CLAIM_NOTE_LEN] {
        [self.value, self.secret_hash]
    }

    fn deserialise(preimage: [Field; CLAIM_NOTE_LEN]) -> Self {
        ClaimNote {
            value: preimage[0],
            secret_hash: preimage[1],
            header: NoteHeader::empty(),
        }
    }

    fn compute_note_hash(self) -> Field {
        dep::std::hash::pedersen([
            self.value, 
            self.secret_hash,
        ])[0]
    }

    fn compute_nullifier(self) -> Field {
        let note_hash_for_nullify = compute_note_hash_for_read_or_nullify(ClaimNoteMethods, self);
        dep::std::hash::pedersen([
            note_hash_for_nullify,
            self.secret_hash, // Include the secret_hash again so that the public won't know the note has been claimed.
        ])[0]
    }

    fn dummy() -> Self {
        ClaimNote {
            value: 0,
            secret_hash: 0,
            header: NoteHeader::empty(),
        }
    }

    fn set_header(&mut self, header: NoteHeader) {
        self.header = header;
    }
}

fn deserialise(preimage: [Field; CLAIM_NOTE_LEN]) -> ClaimNote {
    ClaimNote::deserialise(preimage)
}

fn serialise(note: ClaimNote) -> [Field; CLAIM_NOTE_LEN] {
    note.serialise()
}

fn compute_note_hash(note: ClaimNote) -> Field {
    note.compute_note_hash()
}

fn compute_nullifier(note: ClaimNote) -> Field {
    note.compute_nullifier()
}

fn dummy() -> ClaimNote {
    ClaimNote::dummy()
}

fn get_header(note: ClaimNote) -> NoteHeader {
    note.header
}

fn set_header(note: &mut ClaimNote, header: NoteHeader) {
    note.set_header(header)
}

global ClaimNoteMethods = NoteInterface {
    deserialise,
    serialise,
    compute_note_hash,
    compute_nullifier,
    dummy,
    get_header,
    set_header,
};
