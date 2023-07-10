use dep::std::hash::pedersen;
use dep::aztec::note::note_interface::NoteInterface;

global CLAIM_NOTE_LEN: Field = 2;

struct ClaimNote {
    value: Field,
    secret_hash: Field,
}

impl ClaimNote {
    fn new(value: Field, secret_hash: Field) -> Self {
        ClaimNote {
            value,
            secret_hash,
        }
    }

    fn serialise(self) -> [Field; CLAIM_NOTE_LEN] {
        [self.value, self.secret_hash]
    }

    fn dummy() -> Self {
        ClaimNote {
            value: 0,
            secret_hash: 0,
        }
    }

    fn is_dummy(self) -> bool {
        self.value == 0
    }
}

fn deserialise(preimage: [Field; CLAIM_NOTE_LEN]) -> ClaimNote {
    ClaimNote {
        value: preimage[0],
        secret_hash: preimage[1],
    }
}

fn serialise(note: ClaimNote) -> [Field; CLAIM_NOTE_LEN] {
    note.serialise()
}

fn compute_note_hash(note: ClaimNote) -> Field {
    dep::std::hash::pedersen([
        note.value, 
        note.secret_hash,
    ])[0]
}

fn compute_nullifier(note: ClaimNote, final_note_hash: Field) -> Field {
    dep::std::hash::pedersen([
        final_note_hash,
        note.secret_hash, // Include the secret_hash again so that the public won't know the note has been claimed.
    ])[0]
}

fn dummy() -> ClaimNote {
    ClaimNote::dummy()
}

fn is_dummy(note: ClaimNote) -> bool {
    note.is_dummy()
}

global ClaimNoteInterface = NoteInterface {
    deserialise,
    serialise,
    compute_note_hash,
    compute_nullifier,
    dummy,
    is_dummy,
};
