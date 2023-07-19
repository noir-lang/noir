use dep::aztec::note::{
    note_header::NoteHeader,
    note_interface::NoteInterface,
    utils::compute_siloed_note_hash,
};
use dep::aztec::oracle::{
    rand::rand,
    get_secret_key::get_secret_key,
};
use dep::aztec::types::point::Point;

global VALUE_NOTE_LEN: Field = 5;

struct ValueNote {
    value: Field,
    owner: Point,
    randomness: Field,
    is_real: bool,
    header: NoteHeader,
}

impl ValueNote {
    fn new(value: Field, owner: Point) -> Self {
        let randomness = rand();
        let is_real = value != 0;
        let header = NoteHeader::empty();
        ValueNote {
            value,
            owner,
            randomness,
            is_real,
            header,
        }
    }

    fn serialise(self) -> [Field; VALUE_NOTE_LEN] {
        [self.value, self.owner.x, self.owner.y, self.randomness, self.is_real as Field]
    }

    fn deserialise(preimage: [Field; VALUE_NOTE_LEN]) -> Self {
        ValueNote {
            value: preimage[0],
            owner: Point {
                x: preimage[1],
                y: preimage[2]
            },
            randomness: preimage[3],
            is_real: preimage[4] as bool,
            header: NoteHeader::empty(),
        }
    }

    fn compute_note_hash(self) -> Field {
        dep::std::hash::pedersen([
            self.value, 
            self.owner.x, 
            self.owner.y, 
            self.randomness,
            self.is_real as Field,
        ])[0]
    }

    fn compute_nullifier(self) -> Field {
        if (!self.is_real) {
            0
        } else {
            let siloed_note_hash = compute_siloed_note_hash(ValueNoteInterface, self);
            let secret = get_secret_key(self.owner);
            dep::std::hash::pedersen([
                siloed_note_hash,
                secret, 
            ])[0]
        }
    }

    fn dummy() -> Self {
        ValueNote {
            value: 0,
            owner: Point { x: 0, y: 0 },
            randomness: 0,
            is_real: false,
            header: NoteHeader::empty(),
        }
    }

    fn is_dummy(self) -> bool {
        !self.is_real
    }

    fn set_header(mut self: Self, header: NoteHeader) -> Self {
        self.header = header;
        self
    }

    fn validate(self, sender: Point) {
        let sender_equals = (sender.x == self.owner.x) & (sender.y == self.owner.y);
        assert((!self.is_real) | sender_equals);
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

fn is_dummy(note: ValueNote) -> bool {
    note.is_dummy()
}

fn get_header(note: ValueNote) -> NoteHeader {
    note.header
}

fn set_header(note: ValueNote, header: NoteHeader) -> ValueNote {
    note.set_header(header)
}

global ValueNoteInterface = NoteInterface {
    deserialise,
    serialise,
    compute_note_hash,
    compute_nullifier,
    dummy,
    is_dummy,
    get_header,
    set_header,
};
