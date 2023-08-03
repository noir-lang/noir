use dep::aztec::note::{
    note_header::NoteHeader,
    note_interface::NoteInterface,
    utils::compute_note_hash_for_read_or_nullify,
};
use dep::aztec::oracle::{
    rand::rand,
    get_secret_key::get_secret_key,
    get_public_key::get_public_key,
};
use dep::aztec::types::point::Point;

global VALUE_NOTE_LEN: Field = 3; // 3 plus a header.

struct ValueNote {
    value: Field,
    owner: Field,
    randomness: Field,
    header: NoteHeader,
}

impl ValueNote {
    fn new(value: Field, owner: Field) -> Self {
        let randomness = rand();
        let header = NoteHeader::empty();
        ValueNote {
            value,
            owner,
            randomness,
            header,
        }
    }

    fn serialise(self) -> [Field; VALUE_NOTE_LEN] {
        [self.value, self.owner, self.randomness]
    }

    fn deserialise(preimage: [Field; VALUE_NOTE_LEN]) -> Self {
        ValueNote {
            value: preimage[0],
            owner: preimage[1],
            randomness: preimage[2],
            header: NoteHeader::empty(),
        }
    }

    fn compute_note_hash(self) -> Field {
        dep::std::hash::pedersen([
            self.value, 
            self.owner,
            self.randomness,
        ])[0]
    }

    fn compute_nullifier(self) -> Field {
        let note_hash_for_nullify = compute_note_hash_for_read_or_nullify(ValueNoteMethods, self);
        let owner_nullifying_public_key = get_public_key(self.owner);
        // TODO: get_secret_key should just accept an address
        // TODO!
        let secret = get_secret_key(owner_nullifying_public_key);
        dep::std::hash::pedersen([
            note_hash_for_nullify,
            secret,
        ])[0]
    }

    fn dummy() -> Self {
        ValueNote {
            value: 0,
            owner: 0,
            randomness: 0,
            header: NoteHeader::empty(),
        }
    }

    fn set_header(&mut self, header: NoteHeader) {
        self.header = header;
    }
}

fn deserialise(preimage: [Field; VALUE_NOTE_LEN]) -> ValueNote {
    ValueNote::deserialise(preimage)
}

fn serialise(note: ValueNote) -> [Field; VALUE_NOTE_LEN] {
    note.serialise()
}

fn compute_note_hash(note: ValueNote) -> Field {
    note.compute_note_hash()
}

fn compute_nullifier(note: ValueNote) -> Field {
    note.compute_nullifier()
}

fn dummy() -> ValueNote {
    ValueNote::dummy()
}

fn get_header(note: ValueNote) -> NoteHeader {
    note.header
}

fn set_header(note: &mut ValueNote, header: NoteHeader) {
    note.set_header(header)
}

global ValueNoteMethods = NoteInterface {
    deserialise,
    serialise,
    compute_note_hash,
    compute_nullifier,
    dummy,
    get_header,
    set_header,
};
